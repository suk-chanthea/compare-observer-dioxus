//! File-system watcher — thin wrapper around the `notify` crate.
//! Mirrors the `WatcherThread` from the C++ project.

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tokio::sync::mpsc::UnboundedSender;

/// Events emitted by the watcher and consumed on the Dioxus async runtime.
#[derive(Debug, Clone)]
pub enum WatchEvent {
    Created  { system_index: usize, path: String },
    Modified { system_index: usize, path: String },
    Deleted  { system_index: usize, path: String },
}

// ── Static watcher store ──────────────────────────────────────────────────────
// Watchers are kept alive here; dropping them stops the watch.

static WATCHERS: OnceLock<Mutex<Vec<RecommendedWatcher>>> = OnceLock::new();

fn watcher_store() -> &'static Mutex<Vec<RecommendedWatcher>> {
    WATCHERS.get_or_init(|| Mutex::new(Vec::new()))
}

/// Start watching `source` for system `system_index`.
/// File-change events are sent through `tx`.
pub fn start_watching(
    system_index: usize,
    source: &str,
    tx: UnboundedSender<WatchEvent>,
) -> Result<(), String> {
    let root = PathBuf::from(source);
    if !root.exists() {
        return Err(format!("Source path does not exist: {}", root.display()));
    }

    let root_c = root.clone();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(event) = res else { return };

        // Classify into a simple tag before iterating paths (avoids move issues).
        let tag = match event.kind {
            EventKind::Create(_) => "created",
            EventKind::Remove(_) => "deleted",
            EventKind::Modify(_) => "modified",
            _ => return,
        };

        for abs in &event.paths {
            // Skip directories
            if abs.is_dir() { continue; }

            let rel = abs
                .strip_prefix(&root_c)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| abs.to_string_lossy().replace('\\', "/"));

            if rel.is_empty() { continue; }

            let ev = match tag {
                "created"  => WatchEvent::Created  { system_index, path: rel },
                "deleted"  => WatchEvent::Deleted  { system_index, path: rel },
                _          => WatchEvent::Modified { system_index, path: rel },
            };
            let _ = tx.send(ev);
        }
    })
    .map_err(|e| e.to_string())?;

    watcher
        .watch(&root, RecursiveMode::Recursive)
        .map_err(|e| e.to_string())?;

    watcher_store().lock().unwrap().push(watcher);
    Ok(())
}

/// Drop all active watchers, which stops file monitoring.
pub fn stop_watching() {
    watcher_store().lock().unwrap().clear();
}

/// Returns `true` if `rel_path` matches any rule in `except_rules`.
/// Mirrors `FileWatcherApp::isPathExcluded` from the C++ project.
/// A rule matches when any path segment equals the rule string,
/// e.g. rule ".git" matches "project/.git/config" or ".git".
pub fn is_excluded(rel_path: &str, except_rules: &[String]) -> bool {
    for rule in except_rules {
        let rule = rule.trim();
        if rule.is_empty() { continue; }
        // Check every segment of the forward-slash normalised path
        if rel_path.split('/').any(|seg| seg == rule) {
            return true;
        }
    }
    false
}

/// Count files in `dir` recursively, honouring `except_rules`.
/// Mirrors the first-pass count loop in C++ `startWatching`.
pub fn count_files(dir: &PathBuf, except_rules: &[String]) -> usize {
    let Ok(rd) = std::fs::read_dir(dir) else { return 0 };
    let mut n = 0;
    for entry in rd.flatten() {
        let path = entry.path();
        let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        if is_excluded(&name, except_rules) { continue; }
        if path.is_dir() {
            n += count_files(&path, except_rules);
        } else {
            n += 1;
        }
    }
    n
}

/// Walk `dir` recursively collecting relative file paths — NO file content is read.
/// This is the fast first stage of the two-phase baseline: collect paths instantly,
/// then read content in the background after watching has already started.
pub fn collect_file_paths(
    root: &PathBuf,
    dir: &PathBuf,
    except_rules: &[String],
    out: &mut Vec<String>,
) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"));
        if is_excluded(&rel, except_rules) { continue; }
        if path.is_dir() {
            collect_file_paths(root, &path, except_rules, out);
        } else {
            out.push(rel);
        }
    }
}

/// Maximum file size we capture into the baseline for diff purposes (10 MB).
/// Larger files are recorded with empty content so the path still appears.
const MAX_BASELINE_BYTES: u64 = 10 * 1024 * 1024;

/// Walk `dir` recursively, sending each non-excluded file through `tx` as it is read.
/// Runs inside `tokio::task::spawn_blocking` and sends over a tokio unbounded channel
/// so the async consumer can properly `await` instead of spin-polling.
pub fn capture_baseline_channel(
    root: &PathBuf,
    dir: &PathBuf,
    except_rules: &[String],
    tx: &tokio::sync::mpsc::UnboundedSender<(String, String)>,
) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"));

        if is_excluded(&rel, except_rules) { continue; }

        if path.is_dir() {
            capture_baseline_channel(root, &path, except_rules, tx);
        } else {
            // Skip files that are too large or appear to be binary.
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let content = if size > MAX_BASELINE_BYTES {
                String::new()
            } else {
                std::fs::read_to_string(&path).unwrap_or_default()
            };
            if tx.send((rel, content)).is_err() { return; }
        }
    }
}

/// Original synchronous baseline capture (kept for tests / non-progress paths).
pub fn capture_baseline(root: &PathBuf, dir: &PathBuf) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return out };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(capture_baseline(root, &path));
        } else {
            let rel = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"));
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            out.push((rel, content));
        }
    }
    out
}
