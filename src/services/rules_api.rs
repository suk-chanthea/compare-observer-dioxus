use serde::Deserialize;

use crate::core::rules::{build_default_except, build_default_without, rules_from_vec};

// Embed log_sys.json from the project root at compile time.
// The file lives at <project_root>/log_sys.json and is always
// available in the binary regardless of where it is deployed.
const EMBEDDED_JSON: &str = include_str!("../../log_sys.json");

#[derive(Deserialize)]
struct RulesApiResponse {
    without: Option<Vec<String>>,
    except:  Option<Vec<String>>,
}

/// Resolved rule tables returned after loading rules from any source.
pub struct RemoteRules {
    pub without: Option<Vec<Vec<String>>>,
    pub except:  Option<Vec<Vec<String>>>,
    /// Human-readable description of where the data came from.
    pub source:  String,
}

/// Load exclusion rules using a three-step fallback chain:
///
/// 1. **URL** — try `url` if non-empty (the API endpoint in Settings).
/// 2. **Embedded `log_sys.json`** — the file baked into the binary at
///    compile time from `<project_root>/log_sys.json`.
/// 3. **Built-in code defaults** — hardcoded in `rules.rs` as a last resort.
///
/// Always returns `Ok(RemoteRules)` — the caller never needs to handle failure.
pub async fn fetch_rules(url: &str, cols: usize) -> Result<RemoteRules, String> {
    // ── Step 1: try the URL ───────────────────────────────────────────────────
    if !url.is_empty() {
        if let Ok(rules) = try_url(url, cols).await {
            return Ok(rules);
        }
    }

    // ── Step 2: embedded log_sys.json (compiled into the binary) ─────────────
    if let Ok(data) = serde_json::from_str::<RulesApiResponse>(EMBEDDED_JSON) {
        return Ok(build_result(data, cols, "embedded log_sys.json (project file)".into()));
    }

    // ── Step 3: hardcoded fallback ────────────────────────────────────────────
    Ok(RemoteRules {
        without: Some(build_default_without(cols)),
        except:  Some(build_default_except(cols)),
        source:  "built-in defaults".into(),
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn try_url(url: &str, cols: usize) -> Result<RemoteRules, String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let data = resp
        .json::<RulesApiResponse>()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    if data.without.as_ref().map(|v| v.is_empty()).unwrap_or(true)
        && data.except.as_ref().map(|v| v.is_empty()).unwrap_or(true)
    {
        return Err("empty response".into());
    }

    Ok(build_result(data, cols, format!("URL: {url}")))
}

fn build_result(data: RulesApiResponse, cols: usize, source: String) -> RemoteRules {
    RemoteRules {
        without: data
            .without
            .filter(|v| !v.is_empty())
            .map(|v| rules_from_vec(&v, cols)),
        except: data
            .except
            .filter(|v| !v.is_empty())
            .map(|v| rules_from_vec(&v, cols)),
        source,
    }
}
