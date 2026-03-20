use serde::Deserialize;

use crate::core::rules::rules_from_vec;

#[derive(Deserialize)]
struct RulesApiResponse {
    without: Option<Vec<String>>,
    except: Option<Vec<String>>,
}

/// Resolved rule tables returned after a successful fetch.
pub struct RemoteRules {
    pub without: Option<Vec<Vec<String>>>,
    pub except: Option<Vec<Vec<String>>>,
}

/// Fetch exclusion rules from the remote API.
///
/// `cols` is the current number of system columns so the flat rule list is
/// spread across the right number of columns.
pub async fn fetch_rules(url: &str, cols: usize) -> Result<RemoteRules, String> {
    let resp = reqwest::get(url)
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let data = resp
        .json::<RulesApiResponse>()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    Ok(RemoteRules {
        without: data
            .without
            .filter(|v| !v.is_empty())
            .map(|v| rules_from_vec(&v, cols)),
        except: data
            .except
            .filter(|v| !v.is_empty())
            .map(|v| rules_from_vec(&v, cols)),
    })
}
