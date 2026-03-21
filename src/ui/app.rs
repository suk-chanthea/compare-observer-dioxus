use chrono::Local;
use dioxus::desktop::{use_wry_event_handler, window as desktop_window};
use dioxus::prelude::*;
use std::path::Path;

use crate::core::{
    file_entry::FileEntry,
    settings::{load_settings, save_settings, SettingsData, SystemConfigData, DEFAULT_SYSTEMS},
};
use crate::services::{
    file_watcher::{
        capture_baseline_channel, collect_file_paths, is_excluded, start_watching, stop_watching,
        WatchEvent,
    },
    telegram,
};
use crate::ui::{
    dialogs::{log_dialog::LogDialog, settings_dialog::SettingsDialog},
    styles,
    widgets::file_watcher_table::FileWatcherTable,
};
use crate::utils::file_ops;
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn now_ts() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn add_log(log_entries: &mut Signal<Vec<(String, String)>>, msg: impl Into<String>) {
    log_entries.write().push((now_ts(), msg.into()));
}

/// Extract the except-rule strings for system column `idx`.
/// Mirrors `ruleListForSystem(m_exceptRules, i)` from C++.
fn except_rules_for(settings: &SettingsData, idx: usize) -> Vec<String> {
    settings
        .except_rows
        .iter()
        .filter_map(|row| {
            row.get(idx)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .collect()
}

// ── Progress state (mirrors QProgressDialog) ─────────────────────────────────

#[derive(Clone)]
struct ProgressState {
    title:   String,
    label:   String, // current file name
    current: usize,
    total:   usize,
}

// ── Per-system UI metadata ────────────────────────────────────────────────────

#[derive(Clone)]
struct SystemMeta {
    name: String,
    description: String,
    status: String,
    selected: bool,
}

impl SystemMeta {
    fn from_config(cfg: &SystemConfigData, selected: bool) -> Self {
        Self {
            name: if cfg.name.trim().is_empty() {
                "System".to_string()
            } else {
                cfg.name.clone()
            },
            description: String::new(),
            status: "Idle".to_string(),
            selected,
        }
    }
}

// ── App ───────────────────────────────────────────────────────────────────────

#[component]
pub fn App() -> Element {
    // ── Persistent state ─────────────────────────────────────────────────────
    let mut settings = use_signal(load_settings);

    // ── UI state ──────────────────────────────────────────────────────────────
    let mut show_settings     = use_signal(|| false);
    let mut show_logs         = use_signal(|| false);
    let mut is_watching       = use_signal(|| false);
    let mut progress: Signal<Option<ProgressState>> = use_signal(|| None);
    let mut show_close_confirm = use_signal(|| false);

    // ── Intercept the OS close button ─────────────────────────────────────────
    // When file-watching is active we show a confirmation dialog rather than
    // letting the window close immediately.  `LastWindowHides` (set in main.rs)
    // hides the window on CloseRequested; we re-show it while the dialog is
    // visible, then call window().close() only on explicit user confirmation.
    use_wry_event_handler(move |event, _| {
        use dioxus::desktop::tao::event::{Event, WindowEvent};
        if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
            if *is_watching.read() {
                show_close_confirm.set(true);
                // The Dioxus runtime hides the window after this handler returns
                // (LastWindowHides).  Re-show it so the confirm dialog is visible.
                let win = desktop_window();
                spawn(async move { win.set_visible(true); });
            } else {
                // Not watching — exit the application normally.
                desktop_window().close();
            }
        }
    });

    // ── Systems ───────────────────────────────────────────────────────────────
    let mut systems: Signal<Vec<SystemMeta>> = use_signal(|| {
        let s = load_settings();
        let n = s.systems.len().max(DEFAULT_SYSTEMS);
        (0..n).map(|i| {
            let cfg = s.systems.get(i).cloned().unwrap_or_else(|| SystemConfigData {
                name: format!("System {}", i + 1),
                ..Default::default()
            });
            let sel = s.selected_systems.get(i).copied().unwrap_or(true);
            SystemMeta::from_config(&cfg, sel)
        }).collect()
    });

    // ── Per-system file entries — only CHANGED files shown (like C++) ─────────
    let mut all_entries: Signal<Vec<Vec<FileEntry>>> = use_signal(|| {
        let n = load_settings().systems.len().max(DEFAULT_SYSTEMS);
        vec![vec![]; n]
    });

    // ── Baseline content map — captured silently, never shown in table ────────
    // HashMap<rel_path, file_content> per system index.
    let mut baseline: Signal<Vec<HashMap<String, String>>> = use_signal(|| {
        let n = load_settings().systems.len().max(DEFAULT_SYSTEMS);
        vec![HashMap::new(); n]
    });

    // ── Log entries ───────────────────────────────────────────────────────────
    let mut log_entries: Signal<Vec<(String, String)>> = use_signal(Vec::new);

    // ── Snapshots for rendering (avoids holding read-lock inside RSX) ─────────
    let sys_info: Vec<(usize, String, bool)> = systems
        .read()
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.name.clone(), s.selected))
        .collect();

    let statuses: Vec<String> = systems
        .read()
        .iter()
        .map(|s| format!("• {}: {}", s.name, s.status))
        .collect();

    let panels: Vec<(usize, String, String, SystemConfigData)> = {
        let metas = systems.read();
        let cfgs  = settings.read();
        metas.iter().enumerate()
            .filter(|(_, s)| s.selected)
            .map(|(i, s)| {
                let cfg = cfgs.systems.get(i).cloned().unwrap_or_default();
                (i, s.name.clone(), s.description.clone(), cfg)
            })
            .collect()
    };

    // ── Save selection when a system tab is toggled ───────────────────────────
    let mut save_selection = move || {
        let sel: Vec<bool> = systems.read().iter().map(|m| m.selected).collect();
        let mut s = settings.write();
        s.selected_systems = sel;
        let snap = s.clone();
        drop(s);
        save_settings(&snap);
    };

    rsx! {
        document::Style { {styles::GLOBAL_CSS} }
        div {
            class: "app-root",

            // ── Toolbar (no fake menu-bar — mirrors C++ toolbar only) ─────────
            div {
                class: "toolbar",

                // Left: system selector tabs
                div {
                    class: "toolbar-left",
                    span { class: "label-text", "Select Systems:" }
                    for (i, nm, sel) in sys_info {
                        {
                            let watching = *is_watching.read();
                            rsx! {
                                button {
                                    key: "{i}",
                                    class: if watching {
                                        if sel { "sys-btn sys-btn-on sys-btn-locked" } else { "sys-btn sys-btn-locked" }
                                    } else {
                                        if sel { "sys-btn sys-btn-on" } else { "sys-btn" }
                                    },
                                    disabled: watching,
                                    title: if watching { "Stop watching to change system selection" } else { "" },
                                    onclick: move |_| {
                                        if !*is_watching.read() {
                                            systems.write()[i].selected ^= true;
                                            save_selection();
                                        }
                                    },
                                    if sel { "✓ " } else { "" }
                                    "{nm}"
                                }
                            }
                        }
                    }
                }

                // Right: status dots + action buttons
                div {
                    class: "toolbar-right",
                    for s in &statuses {
                        span { class: "status-dot-label", "{s}" }
                    }

                    // ── Start / Stop Watching ────────────────────────────────
                    button {
                        class: if *is_watching.read() { "btn btn-stop" } else { "btn btn-start" },
                        disabled: progress.read().is_some(),
                        onclick: move |_| {
                            if *is_watching.read() {
                                // ── Stop ──────────────────────────────────────
                                stop_watching();
                                is_watching.set(false);
                                for s in systems.write().iter_mut() {
                                    s.status = "Idle".to_string();
                                }
                                add_log(&mut log_entries, "⏹ Watching stopped");
                            } else {
                                // ── Start (async — mirrors C++ startWatching) ─
                                let settings_snap = settings.read().clone();
                                let n = settings_snap.systems.len();

                                // Resize entries Vec
                                {
                                    let mut ew = all_entries.write();
                                    ew.resize_with(n, Vec::new);
                                    for v in ew.iter_mut() { v.clear(); }
                                }

                                // Check at least one system selected with a source
                                let has_any = settings_snap.systems.iter().enumerate().any(|(i, s)| {
                                    systems.read().get(i).map(|m| m.selected).unwrap_or(false)
                                        && !s.source.is_empty()
                                });
                                if !has_any {
                                    add_log(&mut log_entries, "⚠ No systems selected or source paths empty");
                                    return;
                                }

                                // Show scanning indicator immediately
                                progress.set(Some(ProgressState {
                                    title: "Starting Watchers — Scanning…".into(),
                                    label: "Collecting file list…".into(),
                                    current: 0,
                                    total: 0,
                                }));

                                // ── Two-phase async start ─────────────────────
                                // Phase 1 (fast): collect paths only — no file reads.
                                //   Watching starts as soon as paths are known.
                                // Phase 2 (background): read file content for diffs
                                //   silently after watching is already running.
                                spawn(async move {
                                    let (watch_tx, mut watch_rx) =
                                        tokio::sync::mpsc::unbounded_channel::<WatchEvent>();

                                    let mut any_started = false;
                                    let mut sys_except: Vec<Vec<String>> = Vec::new();
                                    // (path_list, system_index, source_root, except_rules)
                                    let mut bg_work: Vec<(Vec<String>, usize, String, Vec<String>)> = Vec::new();

                                    // ── Phase 1: collect paths + start watching ───────────
                                    for (i, sys) in settings_snap.systems.iter().enumerate() {
                                        if !systems.read().get(i).map(|m| m.selected).unwrap_or(false)
                                            || sys.source.is_empty()
                                        {
                                            sys_except.push(vec![]);
                                            continue;
                                        }
                                        let rules = except_rules_for(&settings_snap, i);
                                        sys_except.push(rules.clone());

                                        let root = std::path::PathBuf::from(&sys.source);
                                        let root_c = root.clone();
                                        let rules_c = rules.clone();

                                        // Fast metadata-only walk — no file content reads
                                        let paths = tokio::task::spawn_blocking(move || {
                                            let mut out = Vec::new();
                                            collect_file_paths(&root_c, &root_c, &rules_c, &mut out);
                                            out
                                        }).await.unwrap_or_default();

                                        let file_count = paths.len();
                                        add_log(&mut log_entries, format!(
                                            "📊 '{}': {} files found", sys.name, file_count
                                        ));

                                        // Pre-populate baseline keys (empty content for now)
                                        let bl_map: HashMap<String, String> = paths
                                            .iter()
                                            .map(|p| (p.clone(), String::new()))
                                            .collect();
                                        if let Some(slot) = baseline.write().get_mut(i) {
                                            *slot = bl_map;
                                        }
                                        if let Some(slot) = all_entries.write().get_mut(i) {
                                            slot.clear();
                                        }

                                        // Start the watcher immediately
                                        match start_watching(i, &sys.source, watch_tx.clone()) {
                                            Ok(_) => {
                                                if let Some(m) = systems.write().get_mut(i) {
                                                    m.status = "Watching".to_string();
                                                }
                                                add_log(&mut log_entries, format!("▶ Watching '{}'", sys.source));
                                                any_started = true;
                                            }
                                            Err(e) => {
                                                add_log(&mut log_entries, format!("❌ Watch error: {e}"));
                                            }
                                        }

                                        bg_work.push((paths, i, sys.source.clone(), rules));
                                    }

                                    progress.set(None);

                                    if any_started {
                                        is_watching.set(true);
                                        add_log(&mut log_entries, "✅ File watching started — all systems ready");

                                        // ── Event processor ───────────────────────
                                        spawn(async move {
                                            while let Some(event) = watch_rx.recv().await {
                                                match event {
                                                    WatchEvent::Created { system_index, path } => {
                                                        let rules = sys_except.get(system_index).map(|v| v.as_slice()).unwrap_or(&[]);
                                                        if is_excluded(&path, rules) { continue; }
                                                        if let Some(sys) = all_entries.write().get_mut(system_index) {
                                                            file_ops::upsert_watch_event(sys, &path, "Created", None);
                                                        }
                                                        add_log(&mut log_entries, format!("🆕 Created: {path}"));
                                                    }
                                                    WatchEvent::Modified { system_index, path } => {
                                                        let rules = sys_except.get(system_index).map(|v| v.as_slice()).unwrap_or(&[]);
                                                        if is_excluded(&path, rules) { continue; }
                                                        let old = baseline.read()
                                                            .get(system_index)
                                                            .and_then(|m| m.get(&path))
                                                            .cloned();
                                                        if let Some(sys) = all_entries.write().get_mut(system_index) {
                                                            file_ops::upsert_watch_event(sys, &path, "Modified", old);
                                                        }
                                                        add_log(&mut log_entries, format!("✏ Modified: {path}"));
                                                    }
                                                    WatchEvent::Deleted { system_index, path } => {
                                                        let rules = sys_except.get(system_index).map(|v| v.as_slice()).unwrap_or(&[]);
                                                        if is_excluded(&path, rules) { continue; }
                                                        if let Some(sys) = all_entries.write().get_mut(system_index) {
                                                            file_ops::remove_entry(sys, &path);
                                                        }
                                                        add_log(&mut log_entries, format!("🗑 Deleted: {path}"));
                                                    }
                                                }
                                            }
                                        });

                                        // ── Phase 2: background content loader ────
                                        // Read file content silently after watching has
                                        // started so the diff feature works without
                                        // blocking the initial watch startup.
                                        spawn(async move {
                                            for (paths, sys_idx, source, rules) in bg_work {
                                                let root = std::path::PathBuf::from(&source);
                                                let (ptx, mut prx) =
                                                    tokio::sync::mpsc::unbounded_channel::<(String, String)>();
                                                let root_c = root.clone();
                                                let scan = tokio::task::spawn_blocking(move || {
                                                    capture_baseline_channel(&root_c, &root_c, &rules, &ptx);
                                                });
                                                let mut loaded = 0usize;
                                                while let Some((rel, content)) = prx.recv().await {
                                                    if let Some(slot) = baseline.write().get_mut(sys_idx) {
                                                        slot.insert(rel, content);
                                                    }
                                                    loaded += 1;
                                                }
                                                scan.await.ok();
                                                add_log(&mut log_entries, format!(
                                                    "📖 Baseline content loaded: {loaded} files (diff ready)"
                                                ));
                                                let _ = paths; // suppress unused warning
                                            }
                                        });
                                    }
                                });
                            }
                        },
                        if *is_watching.read() { "Stop Watching" } else { "Start Watching" }
                    }

                    // ── View Logs ────────────────────────────────────────────
                    button {
                        class: "btn",
                        onclick: move |_| show_logs.set(true),
                        "View Logs"
                    }

                    // ── Settings ─────────────────────────────────────────────
                    button {
                        class: "btn",
                        onclick: move |_| show_settings.set(true),
                        "⚙"
                    }
                }
            }

            // ── Main content — one panel per selected system ──────────────────
            div {
                class: "main-content",
                for (i, nm, desc, cfg) in panels {
                    SystemPanel {
                        key: "{i}",
                        index: i,
                        name: nm,
                        description: desc,
                        system_config: cfg,
                        all_entries,
                        baseline,
                        log_entries,
                        telegram_token: settings.read().telegram_token.clone(),
                        telegram_chat_id: settings.read().telegram_chat_id.clone(),
                        username: settings.read().username.clone(),
                        notifications_enabled: settings.read().notifications_enabled,
                        on_description_change: move |val: String| {
                            systems.write()[i].description = val;
                        },
                    }
                }
            }

            // ── Dialogs ───────────────────────────────────────────────────────
            if *show_settings.read() {
                SettingsDialog {
                    username:             settings.read().username.clone(),
                    api_url:              settings.read().api_url.clone(),
                    telegram_token:       settings.read().telegram_token.clone(),
                    telegram_chat_id:     settings.read().telegram_chat_id.clone(),
                    notifications_enabled: settings.read().notifications_enabled,
                    systems:             settings.read().systems.clone(),
                    without_rows:        settings.read().without_rows.clone(),
                    except_rows:         settings.read().except_rows.clone(),
                    on_save: move |data: SettingsData| {
                        let new_count = data.systems.len();
                        // Sync system metas
                        let mut metas = systems.write();
                        metas.resize_with(new_count, || SystemMeta {
                            name: String::new(),
                            description: String::new(),
                            status: "Idle".to_string(),
                            selected: true,
                        });
                        for (meta, cfg) in metas.iter_mut().zip(data.systems.iter()) {
                            if !cfg.name.trim().is_empty() {
                                meta.name = cfg.name.clone();
                            }
                            if meta.status.is_empty() {
                                meta.status = "Idle".to_string();
                                meta.selected = true;
                            }
                        }
                        // Preserve current selection in saved data
                        let sel: Vec<bool> = metas.iter().map(|m| m.selected).collect();
                        drop(metas);

                        // Resize entries + baseline vecs
                        let mut ew = all_entries.write();
                        ew.resize_with(new_count, Vec::new);
                        drop(ew);
                        let mut bw = baseline.write();
                        bw.resize_with(new_count, HashMap::new);
                        drop(bw);

                        let mut to_save = data.clone();
                        to_save.selected_systems = sel;
                        save_settings(&to_save);
                        settings.set(to_save);
                        show_settings.set(false);
                    },
                    on_cancel: move |_| show_settings.set(false),
                }
            }

            if *show_logs.read() {
                LogDialog {
                    entries: log_entries,
                    on_close: move |_| show_logs.set(false),
                }
            }

            // ── Progress overlay (shown while capturing baseline) ─────────────
            if let Some(prog) = progress.read().as_ref() {
                ProgressOverlay {
                    title:   prog.title.clone(),
                    label:   prog.label.clone(),
                    current: prog.current,
                    total:   prog.total,
                }
            }

            // ── Close-while-watching confirmation ─────────────────────────────
            if *show_close_confirm.read() {
                CloseConfirmDialog {
                    on_confirm: move |_| {
                        show_close_confirm.set(false);
                        desktop_window().close();
                    },
                    on_cancel: move |_| show_close_confirm.set(false),
                }
            }
        }
    }
}

// ── ProgressOverlay ───────────────────────────────────────────────────────────

#[component]
fn ProgressOverlay(
    title:   String,
    label:   String,
    current: usize,
    total:   usize,
) -> Element {
    let (pct, indeterminate) = if total > 0 {
        ((current * 100 / total).min(100), false)
    } else {
        (0, true)
    };

    rsx! {
        div {
            class: "progress-overlay",

            div {
                class: "progress-dialog",

                div { class: "progress-title", "{title}" }

                // Bar + percentage label
                div { class: "progress-bar-row",
                    div {
                        class: "progress-bar-track",
                        div {
                            class: "progress-bar-fill",
                            style: if indeterminate {
                                "width: 40%; animation: pulse-slide 1.4s ease-in-out infinite;".into()
                            } else {
                                format!("width: {pct}%;")
                            },
                        }
                    }
                    if !indeterminate {
                        span { class: "progress-pct", "{pct}%" }
                    }
                }

                // Current file name
                div { class: "progress-label", "{label}" }

                // Counts
                if !indeterminate {
                    div { class: "progress-counts", "{current} / {total} files" }
                } else {
                    div { class: "progress-counts", "Scanning…" }
                }
            }
        }
    }
}

// ── SystemPanel ───────────────────────────────────────────────────────────────

#[component]
fn SystemPanel(
    index: usize,
    name: String,
    description: String,
    system_config: SystemConfigData,
    all_entries: Signal<Vec<Vec<FileEntry>>>,
    baseline: Signal<Vec<HashMap<String, String>>>,
    log_entries: Signal<Vec<(String, String)>>,
    telegram_token: String,
    telegram_chat_id: String,
    username: String,
    notifications_enabled: bool,
    on_description_change: EventHandler<String>,
) -> Element {
    // ── Local dialog state ────────────────────────────────────────────────────
    let mut diff_state:   Signal<Option<DiffState>>   = use_signal(|| None);
    let mut assign_state: Signal<Option<AssignState>> = use_signal(|| None);
    let mut alert_msg:    Signal<Option<String>>      = use_signal(|| None);

    rsx! {
        div {
            class: "system-panel",

            // ── Description row ───────────────────────────────────────────────
            div {
                class: "system-desc-row",
                label { class: "system-desc-label", "Description for {name}:" }
                input {
                    r#type: "text",
                    class: "system-desc-input",
                    value: "{description}",
                    placeholder: "Enter description here...",
                    oninput: move |e| on_description_change.call(e.value()),
                }
            }

            // ── Table + action buttons ────────────────────────────────────────
            div {
                class: "system-body",
                div {
                    class: "system-table-area",
                    FileWatcherTable {
                        all_entries,
                        system_index: index,
                        on_view_diff: {
                            let cfg = system_config.clone();
                            move |path: String| {
                                // Look up baseline content (captured at watch-start)
                                let old_content = baseline.read()
                                    .get(index)
                                    .and_then(|m| m.get(&path))
                                    .cloned()
                                    .unwrap_or_default();
                                let source = cfg.source.clone();
                                let path_c = path.clone();
                                spawn(async move {
                                    let abs = Path::new(&source).join(&path_c);
                                    let new_content = tokio::fs::read_to_string(&abs)
                                        .await
                                        .unwrap_or_else(|_| "(file not readable)".into());
                                    diff_state.set(Some(DiffState {
                                        title: path_c,
                                        old_content,
                                        new_content,
                                    }));
                                });
                            }
                        },
                    }
                }

                // ── Copy / Copy Send / Assign To ──────────────────────────────
                div {
                    class: "system-actions",

                    // Copy
                    button {
                        class: "btn-action btn-copy",
                        onclick: {
                            let cfg    = system_config.clone();
                            let nm     = name.clone();
                            let mut le = log_entries;
                            move |_| {
                                let files = checked_file_paths(all_entries, index);
                                if files.is_empty() {
                                    alert_msg.set(Some(format!(
                                        "No files selected.\n\nPlease check at least one file in the \"{nm}\" table before copying."
                                    )));
                                    return;
                                }
                                if cfg.destination.is_empty() {
                                    alert_msg.set(Some(format!(
                                        "No copy destination set.\n\nPlease open Settings and fill in the Destination path for \"{nm}\"."
                                    )));
                                    return;
                                }
                                let (ok, fail) = do_copy(&cfg, &files, &mut le, &nm);
                                add_log(&mut le, format!("{nm}: Copy done — ✓{ok} ✗{fail}"));
                                if let Some(sys) = all_entries.write().get_mut(index) {
                                    file_ops::uncheck_paths(sys, &files);
                                }
                            }
                        },
                        "Copy"
                    }

                    // Copy Send
                    button {
                        class: "btn-action btn-copy-send",
                        onclick: {
                            let cfg      = system_config.clone();
                            let nm       = name.clone();
                            let usr      = username.clone();
                            let tok      = telegram_token.clone();
                            let cid      = telegram_chat_id.clone();
                            let desc_val = description.clone();
                            let mut le   = log_entries;
                            move |_| {
                                let files = checked_file_paths(all_entries, index);
                                if files.is_empty() {
                                    alert_msg.set(Some(format!(
                                        "No files selected.\n\nPlease check at least one file in the \"{nm}\" table before copying."
                                    )));
                                    return;
                                }
                                if cfg.destination.is_empty() {
                                    alert_msg.set(Some(format!(
                                        "No copy destination set.\n\nPlease open Settings and fill in the Destination path for \"{nm}\"."
                                    )));
                                    return;
                                }
                                let (ok, _fail) = do_copy(&cfg, &files, &mut le, &nm);
                                add_log(&mut le, format!("{nm}: Copy+Send — ✓{ok}"));
                                if let Some(sys) = all_entries.write().get_mut(index) {
                                    file_ops::uncheck_paths(sys, &files);
                                }
                                let tok2 = tok.clone();
                                let cid2 = cid.clone();
                                let msg  = telegram::build_notification(&nm, &usr, &desc_val, &files);
                                let mut le2 = le;
                                spawn(async move {
                                    match telegram::send_message(&tok2, &cid2, &msg).await {
                                        Ok(_)  => add_log(&mut le2, "✅ Telegram notification sent"),
                                        Err(e) => add_log(&mut le2, format!("❌ Telegram error: {e}")),
                                    }
                                });
                            }
                        },
                        "Copy Send"
                    }

                    // Assign To
                    button {
                        class: "btn-action btn-assign",
                        onclick: {
                            let nm     = name.clone();
                            let mut le = log_entries;
                            move |_| {
                                let files = checked_file_paths(all_entries, index);
                                if files.is_empty() {
                                    alert_msg.set(Some(format!(
                                        "No files selected.\n\nPlease check at least one file in the \"{nm}\" table before assigning."
                                    )));
                                    add_log(&mut le, format!("⚠ {nm}: no files checked"));
                                    return;
                                }
                                assign_state.set(Some(AssignState {
                                    files,
                                    name_input: String::new(),
                                    desc_input: String::new(),
                                }));
                            }
                        },
                        "Assign To"
                    }
                }
            }

            // ── Diff overlay ──────────────────────────────────────────────────
            if let Some(diff) = diff_state.read().as_ref() {
                DiffDialog {
                    title: diff.title.clone(),
                    old_content: diff.old_content.clone(),
                    new_content: diff.new_content.clone(),
                    on_close: move |_| diff_state.set(None),
                }
            }

            // ── Assign To overlay ─────────────────────────────────────────────
            if assign_state.read().is_some() {
                AssignDialog {
                    state: assign_state,
                    system_config: system_config.clone(),
                    system_name: name.clone(),
                    all_entries,
                    system_index: index,
                    log_entries,
                    on_close: move |_| assign_state.set(None),
                }
            }

            // ── Alert popup ───────────────────────────────────────────────────
            if let Some(msg) = alert_msg.read().clone() {
                AlertDialog {
                    message: msg,
                    on_close: move |_| alert_msg.set(None),
                }
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn checked_file_paths(all_entries: Signal<Vec<Vec<FileEntry>>>, index: usize) -> Vec<String> {
    all_entries
        .read()
        .get(index)
        .map(|sys| file_ops::checked_paths(sys))
        .unwrap_or_default()
}

/// Copy files from source to destination (and git if configured).
/// Returns (success_count, fail_count).
fn do_copy(
    cfg: &SystemConfigData,
    files: &[String],
    le: &mut Signal<Vec<(String, String)>>,
    label: &str,
) -> (usize, usize) {
    let mut ok = 0usize;
    let mut fail = 0usize;

    for rel in files {
        let src = Path::new(&cfg.source).join(rel);
        if !src.exists() {
            add_log(le, format!("❌ {label}: source not found: {rel}"));
            fail += 1;
            continue;
        }

        let mut any_ok = false;

        // Copy to destination
        if !cfg.destination.is_empty() {
            let dst = Path::new(&cfg.destination).join(rel);
            if let Some(p) = dst.parent() { let _ = std::fs::create_dir_all(p); }
            match std::fs::copy(&src, &dst) {
                Ok(_)  => { any_ok = true; }
                Err(e) => { add_log(le, format!("❌ {label}: copy to dest failed for {rel}: {e}")); }
            }
        }

        // Mirror to git path
        if !cfg.git.is_empty() {
            let gdst = Path::new(&cfg.git).join(rel);
            if let Some(p) = gdst.parent() { let _ = std::fs::create_dir_all(p); }
            let _ = std::fs::copy(&src, &gdst);
        }

        if any_ok || (!cfg.destination.is_empty()) {
            ok += 1;
        } else {
            fail += 1;
        }
    }
    (ok, fail)
}

// ── DiffState / DiffDialog ────────────────────────────────────────────────────

#[derive(Clone)]
struct DiffState {
    title:       String,
    old_content: String,
    new_content: String,
}

// A single diffed line for display
#[derive(Clone, PartialEq)]
enum DiffKind { Same, Added, Removed, Changed }

#[derive(Clone)]
struct DiffLine {
    old_no:  Option<usize>,  // line number in old file (1-based)
    new_no:  Option<usize>,  // line number in new file (1-based)
    old_txt: String,
    new_txt: String,
    kind:    DiffKind,
}

/// Myers-style LCS diff — returns (old_lines, new_lines) paired up with a kind.
fn compute_diff(old: &str, new: &str) -> Vec<DiffLine> {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let n = old_lines.len();
    let m = new_lines.len();

    // Simple DP LCS table
    let mut dp = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if old_lines[i] == new_lines[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    let (mut oi, mut ni) = (1usize, 1usize);  // 1-based line counters

    while i < n || j < m {
        if i < n && j < m && old_lines[i] == new_lines[j] {
            result.push(DiffLine { old_no: Some(oi), new_no: Some(ni),
                old_txt: old_lines[i].to_string(), new_txt: new_lines[j].to_string(),
                kind: DiffKind::Same });
            i += 1; j += 1; oi += 1; ni += 1;
        } else if i < n && (j >= m || dp[i + 1][j] >= dp[i][j + 1]) {
            // Check if the next new line matches — pair as Changed
            if j < m && dp[i + 1][j + 1] == dp[i + 1][j] && dp[i + 1][j + 1] == dp[i][j + 1] {
                result.push(DiffLine { old_no: Some(oi), new_no: Some(ni),
                    old_txt: old_lines[i].to_string(), new_txt: new_lines[j].to_string(),
                    kind: DiffKind::Changed });
                i += 1; j += 1; oi += 1; ni += 1;
            } else {
                result.push(DiffLine { old_no: Some(oi), new_no: None,
                    old_txt: old_lines[i].to_string(), new_txt: String::new(),
                    kind: DiffKind::Removed });
                i += 1; oi += 1;
            }
        } else {
            result.push(DiffLine { old_no: None, new_no: Some(ni),
                old_txt: String::new(), new_txt: new_lines[j].to_string(),
                kind: DiffKind::Added });
            j += 1; ni += 1;
        }
    }
    result
}

/// Indices of change-group starts (first line of each Added/Removed/Changed run)
fn change_positions(lines: &[DiffLine]) -> Vec<usize> {
    let mut pos = Vec::new();
    let mut in_change = false;
    for (i, l) in lines.iter().enumerate() {
        if l.kind != DiffKind::Same {
            if !in_change { pos.push(i); in_change = true; }
        } else {
            in_change = false;
        }
    }
    pos
}

#[component]
fn DiffDialog(
    title: String,
    old_content: String,
    new_content: String,
    on_close: EventHandler<()>,
) -> Element {
    let lines      = compute_diff(&old_content, &new_content);
    let changes    = change_positions(&lines);
    let n_changes  = changes.len();
    let mut cur_change: Signal<usize> = use_signal(|| 0);

    // Sync-scroll: when left pane scrolls, mirror to right and vice-versa
    let sync_js = r#"
        (function(){
            var L = document.getElementById('diff-left');
            var R = document.getElementById('diff-right');
            if(!L||!R) return;
            L.addEventListener('scroll', function(){ R.scrollTop = L.scrollTop; });
            R.addEventListener('scroll', function(){ L.scrollTop = R.scrollTop; });
        })();
    "#;

    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "dialog diff-dialog",
                onclick: move |e| e.stop_propagation(),

                // Header
                div { class: "diff-header",
                    span { class: "diff-title", "{title}" }
                    div { class: "diff-header-right",
                        {
                            let change_label = if n_changes == 0 {
                                "No changes".to_string()
                            } else if n_changes == 1 {
                                "1 change".to_string()
                            } else {
                                format!("{n_changes} changes")
                            };
                            rsx! { span { class: "diff-change-count", "{change_label}" } }
                        }
                        if n_changes > 0 {
                            button {
                                class: "diff-nav-btn",
                                title: "Previous change",
                                onclick: {
                                    let changes2 = changes.clone();
                                    move |_| {
                                        let c = *cur_change.read();
                                        let next = if c == 0 { changes2.len().saturating_sub(1) } else { c - 1 };
                                        cur_change.set(next);
                                        let idx = changes2[next];
                                        let js = format!(
                                            r#"(function(){{var el=document.getElementById('dl-{idx}');if(el){{el.scrollIntoView({{block:'center'}});var o=document.getElementById('dr-{idx}');if(o)o.scrollIntoView({{block:'center'}});}}}})()"#
                                        );
                                        document::eval(&js);
                                    }
                                },
                                "▲"
                            }
                            button {
                                class: "diff-nav-btn",
                                title: "Next change",
                                onclick: {
                                    let changes3 = changes.clone();
                                    move |_| {
                                        let c = *cur_change.read();
                                        let next = (c + 1) % changes3.len().max(1);
                                        cur_change.set(next);
                                        let idx = changes3[next];
                                        let js = format!(
                                            r#"(function(){{var el=document.getElementById('dl-{idx}');if(el){{el.scrollIntoView({{block:'center'}});var o=document.getElementById('dr-{idx}');if(o)o.scrollIntoView({{block:'center'}});}}}})()"#
                                        );
                                        document::eval(&js);
                                    }
                                },
                                "▼"
                            }
                        }
                        button {
                            class: "log-close-btn",
                            onclick: move |_| on_close.call(()),
                            "✕"
                        }
                    }
                }

                // Column headers
                div { class: "diff-col-headers",
                    div { class: "diff-col-hdr old", "Old (Baseline)" }
                    div { class: "diff-col-hdr new", "New (Current)" }
                }

                // Split panes
                div { class: "diff-body",

                    // ── LEFT pane (old) ──────────────────────────────────────────
                    div { id: "diff-left", class: "diff-pane",
                        table { class: "diff-table",
                            tbody {
                                for (row_idx, line) in lines.iter().enumerate() {
                                    {
                                        let row_class = match line.kind {
                                            DiffKind::Removed => "diff-row diff-removed",
                                            DiffKind::Changed => "diff-row diff-changed",
                                            DiffKind::Added   => "diff-row diff-empty",
                                            DiffKind::Same    => "diff-row",
                                        };
                                        let lno = line.old_no.map(|n| n.to_string()).unwrap_or_default();
                                        let txt = line.old_txt.clone();
                                        rsx! {
                                            tr {
                                                id: "dl-{row_idx}",
                                                class: "{row_class}",
                                                td { class: "diff-lno", "{lno}" }
                                                td { class: "diff-code", pre { "{txt}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // ── Divider ──────────────────────────────────────────────────
                    div { class: "diff-divider" }

                    // ── RIGHT pane (new) ─────────────────────────────────────────
                    div { id: "diff-right", class: "diff-pane",
                        table { class: "diff-table",
                            tbody {
                                for (row_idx, line) in lines.iter().enumerate() {
                                    {
                                        let row_class = match line.kind {
                                            DiffKind::Added   => "diff-row diff-added",
                                            DiffKind::Changed => "diff-row diff-changed",
                                            DiffKind::Removed => "diff-row diff-empty",
                                            DiffKind::Same    => "diff-row",
                                        };
                                        let lno = line.new_no.map(|n| n.to_string()).unwrap_or_default();
                                        let txt = line.new_txt.clone();
                                        rsx! {
                                            tr {
                                                id: "dr-{row_idx}",
                                                class: "{row_class}",
                                                td { class: "diff-lno", "{lno}" }
                                                td { class: "diff-code", pre { "{txt}" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Footer
                div { class: "dialog-footer",
                    button {
                        class: "btn",
                        onclick: move |_| on_close.call(()),
                        "Close"
                    }
                }

                // Attach sync-scroll listeners once on mount
                script { dangerous_inner_html: "{sync_js}" }
                // Scroll to first change on open
                if n_changes > 0 {
                    {
                        let first_idx = changes[0];
                        let js = format!(
                            r#"(function(){{var el=document.getElementById('dl-{first_idx}');if(el){{el.scrollIntoView({{block:'center'}});var o=document.getElementById('dr-{first_idx}');if(o)o.scrollIntoView({{block:'center'}});}}}})()"#
                        );
                        rsx! { script { dangerous_inner_html: "{js}" } }
                    }
                }
            }
        }
    }
}

// ── AssignState / AssignDialog ────────────────────────────────────────────────

#[derive(Clone)]
struct AssignState {
    files:      Vec<String>,
    name_input: String,
    desc_input: String,
}

#[component]
fn AssignDialog(
    state: Signal<Option<AssignState>>,
    system_config: SystemConfigData,
    system_name: String,
    all_entries: Signal<Vec<Vec<FileEntry>>>,
    system_index: usize,
    log_entries: Signal<Vec<(String, String)>>,
    on_close: EventHandler<()>,
) -> Element {
    let mut le = log_entries;

    // Read current inputs from state
    let (files, name_val, desc_val) = {
        let s = state.read();
        let st = s.as_ref().unwrap();
        (st.files.clone(), st.name_input.clone(), st.desc_input.clone())
    };

    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "dialog assign-dialog",
                onclick: move |e| e.stop_propagation(),

                h3 { "Assign To" }

                div {
                    label { "Folder Name (required):" }
                    input {
                        r#type: "text",
                        value: "{name_val}",
                        placeholder: "e.g. release-v1.2",
                        oninput: move |e| {
                            if let Some(st) = state.write().as_mut() {
                                st.name_input = e.value();
                            }
                        },
                        autofocus: true,
                    }
                }

                div {
                    label { "Description (optional):" }
                    textarea {
                        value: "{desc_val}",
                        placeholder: "Enter notes or description...",
                        oninput: move |e| {
                            if let Some(st) = state.write().as_mut() {
                                st.desc_input = e.value();
                            }
                        },
                    }
                }

                div {
                    style: "font-size: 12px; color: #888;",
                    "Files to assign: {files.len()}"
                }

                div {
                    class: "mini-dialog-buttons",

                    button {
                        class: "btn btn-copy",
                        onclick: {
                            let cfg   = system_config.clone();
                            let nm    = system_name.clone();
                            move |_| {
                                let st = state.read();
                                let info = st.as_ref().unwrap();
                                let folder_name = info.name_input.trim().to_string();
                                if folder_name.is_empty() {
                                    add_log(&mut le, "⚠ Assign To: folder name is required");
                                    return;
                                }
                                let desc = info.desc_input.trim().to_string();
                                let files_snap = info.files.clone();
                                drop(st);

                                if cfg.assign.is_empty() {
                                    add_log(&mut le, format!("❌ {nm}: assign path not configured"));
                                    on_close.call(());
                                    return;
                                }

                                let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                                let dest_root = Path::new(&cfg.assign)
                                    .join(&folder_name)
                                    .join(&timestamp);

                                let _ = std::fs::create_dir_all(&dest_root);

                                // Write description.txt
                                if !desc.is_empty() {
                                    let _ = std::fs::write(dest_root.join("description.txt"), &desc);
                                }

                                let mut ok = 0usize;
                                let mut fail = 0usize;
                                for rel in &files_snap {
                                    let src = Path::new(&cfg.source).join(rel);
                                    let dst = dest_root.join(rel);
                                    if let Some(p) = dst.parent() { let _ = std::fs::create_dir_all(p); }
                                    match std::fs::copy(&src, &dst) {
                                        Ok(_)  => ok += 1,
                                        Err(e) => {
                                            add_log(&mut le, format!("❌ Assign error {rel}: {e}"));
                                            fail += 1;
                                        }
                                    }
                                }

                                add_log(&mut le, format!(
                                    "📦 {nm}: Assigned {ok} file(s) to '{folder_name}/{timestamp}' ✗{fail}"
                                ));

                                // Remove assigned entries from table
                                if let Some(sys) = all_entries.write().get_mut(system_index) {
                                    sys.retain(|e| !files_snap.contains(&e.path));
                                }

                                on_close.call(());
                            }
                        },
                        "OK"
                    }

                    button {
                        class: "btn",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                }
            }
        }
    }
}

// ── AlertDialog ───────────────────────────────────────────────────────────────

#[component]
fn AlertDialog(
    message:  String,
    on_close: EventHandler<()>,
) -> Element {
    // Split on \n so we can render each line as its own paragraph
    let lines: Vec<String> = message.split('\n').map(|l| l.to_string()).collect();

    rsx! {
        div {
            class: "dialog-overlay alert-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "alert-dialog",
                onclick: move |e| e.stop_propagation(),

                div { class: "alert-icon", "⚠" }

                div { class: "alert-body",
                    for line in lines.iter() {
                        if line.is_empty() {
                            div { style: "height: 8px;" }
                        } else {
                            p { class: "alert-line", "{line}" }
                        }
                    }
                }

                div { class: "alert-footer",
                    button {
                        class: "btn btn-primary alert-ok",
                        onclick: move |_| on_close.call(()),
                        "OK"
                    }
                }
            }
        }
    }
}

// ── CloseConfirmDialog ────────────────────────────────────────────────────────

#[component]
fn CloseConfirmDialog(
    on_confirm: EventHandler<()>,
    on_cancel:  EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "dialog-overlay close-confirm-overlay",

            div {
                class: "close-confirm-dialog",
                onclick: move |e| e.stop_propagation(),

                div { class: "close-confirm-icon", "⚠" }

                p { class: "close-confirm-title", "Stop watching and close?" }
                p { class: "close-confirm-body",
                    "File watching is still active.\n\
                     Closing now will stop all watchers.\n\
                     Are you sure you want to exit?"
                }

                div { class: "close-confirm-footer",
                    button {
                        class: "btn btn-danger ",
                        style: "text-align: center;min-width: 50px;",
                        onclick: move |_| on_confirm.call(()),
                        "Ok"
                    }
                    button {
                        class: "btn",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                }
            }
        }
    }
}
