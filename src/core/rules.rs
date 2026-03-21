// Default rule data — taken directly from log_sys.json.
// Used when the API URL is unreachable and no local file is found.

const DEFAULT_WITHOUT: &[&str] = &[
    "config",
    "config/include",
    "config/include/lang",
    "config/include/title",
    "config/include/title/seo",
    "config/include/header",
    "config/include/footer",
    "config/include/contact_us",
    "config/include/landing",
    "config/include/slider",
    "config/include/page_name",
    "config/include/widget",
    "config/include/sitemap",
    "config/include/robots",
    "mod/index",
    "mod/register",
    "mod/login",
    "mod/message",
    "mod/games",
    "mod/games/lottery",
    "mod/games/lottery/submitter",
    "mod/games/poker",
    "mod/games/hot_games",
    "mod/games/hot_games_lobby",
    "mod/games/ongdo_lobby",
    "mod/games/sports_lobby",
    "mod/games/casino_lobby",
    "mod/games/slots_lobby",
    "mod/games/number_lobby",
    "mod/games/lottery_lobby",
    "mod/games/poker_lobby",
    "mod/games/cockfight",
    "mod/games/cock_lobby",
    "mod/games/fishing_lobby",
    "mod/games/slots",
    "mod/gift",
    "mod/member",
    "mod/member/info",
    "mod/member/reports",
    "mod/member/vip_bonus",
    "mod/member/fix_bank",
    "mod/member/deposit",
    "mod/member/usdt",
    "mod/member/deposit/online",
    "mod/member/deposit/qrcode",
    "mod/member/deposit/depo_menu",
    "mod/member/deposit/submitter",
    "mod/register/submitter",
    "mod/member/with",
    "mod/member/with/with_menu",
    "mod/member/bonus",
    "mod/member/bonus/bonus_menu",
    "mod/member/bonus/submitter",
    "mod/member/nav_bar",
    "mod/games/slots/slot_ajax",
    "mod/member/transfer",
    "mod/member/transfer/submitter",
    "promotions",
    "other",
];

const DEFAULT_EXCEPT: &[&str] = &[
    "content",
    ".git",
    ".idea",
    "index.php",
    "test",
    "test_2",
    ".gitignore",
];

/// Default "Without" exclusion rows, broadcast across `cols` system columns.
pub fn build_default_without(cols: usize) -> Vec<Vec<String>> {
    DEFAULT_WITHOUT
        .iter()
        .map(|&s| vec![s.to_string(); cols.max(1)])
        .collect()
}

/// Default "Except" exclusion rows, broadcast across `cols` system columns.
pub fn build_default_except(cols: usize) -> Vec<Vec<String>> {
    DEFAULT_EXCEPT
        .iter()
        .map(|&s| vec![s.to_string(); cols.max(1)])
        .collect()
}

/// Converts a flat list of rule strings into a rows×cols table
/// (mirrors `SettingsDialog::rulesFromJson`).
pub fn rules_from_vec(entries: &[String], cols: usize) -> Vec<Vec<String>> {
    entries
        .iter()
        .map(|entry| vec![entry.clone(); cols.max(1)])
        .collect()
}
