/// Global application stylesheet — dark theme.
/// Mirrors `Styles::getMainStylesheet()` from styles.cpp.
pub const GLOBAL_CSS: &str = r#"
*, *::before, *::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body, html {
    height: 100%;
    font-family: "Segoe UI", system-ui, sans-serif;
    font-size: 14px;
    background-color: #121212;
    color: #E5E5E5;
}

/* ── App shell ─────────────────────────────────────────── */
.app-root {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #121212;
}

.toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 14px;
    background-color: #1C1C1C;
    border-bottom: 1px solid #2A2A2A;
}

.toolbar-title {
    flex: 1;
    font-weight: 700;
    font-size: 15px;
    color: #EDEDED;
}

.main-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
}

/* ── Buttons ────────────────────────────────────────────── */
.btn {
    background-color: #2A2A2A;
    color: #FAFAFA;
    border: 1px solid #3A3A3A;
    border-radius: 4px;
    padding: 6px 12px;
    font-weight: 600;
    cursor: pointer;
    font-size: 13px;
}
.btn:hover  { background-color: #3A3A3A; }
.btn:active { background-color: #4A4A4A; }

.btn-danger {
    background-color: #C62828;
    color: #fff;
    border: none;
    border-radius: 4px;
    padding: 6px 10px;
    font-size: 15px;
    min-width: 32px;
    max-width: 32px;
    cursor: pointer;
}
.btn-danger:hover  { background-color: #D32F2F; }
.btn-danger:active { background-color: #B71C1C; }

/* ── Text inputs ─────────────────────────────────────────── */
input[type="text"],
input[type="password"] {
    background-color: #1B1B1B;
    color: #F3F3F3;
    border: 1px solid #343434;
    border-radius: 4px;
    padding: 6px 8px;
    font-size: 13px;
    width: 100%;
    outline: none;
}
input[type="text"]:focus,
input[type="password"]:focus {
    border-color: #0B57D0;
    background-color: #222222;
}

/* ── Checkbox ─────────────────────────────────────────────── */
.checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    color: #E1E1E1;
    cursor: pointer;
    user-select: none;
}
.checkbox-label input[type="checkbox"] {
    appearance: none;
    -webkit-appearance: none;
    width: 18px;
    height: 18px;
    border: 1px solid #333;
    border-radius: 3px;
    background-color: #2A2A2A;
    cursor: pointer;
    position: relative;
    flex-shrink: 0;
}
.checkbox-label input[type="checkbox"]:checked {
    background-color: #0B57D0;
    border-color: #0B57D0;
}
.checkbox-label input[type="checkbox"]:checked::after {
    content: "";
    position: absolute;
    left: 3px; top: 1px;
    width: 10px; height: 6px;
    border-left: 2px solid #fff;
    border-bottom: 2px solid #fff;
    transform: rotate(-45deg);
}
.checkbox-label input[type="checkbox"]:indeterminate {
    background-color: #2A2A2A;
    border-color: #0B57D0;
}
.checkbox-label input[type="checkbox"]:indeterminate::after {
    content: "";
    position: absolute;
    left: 3px; top: 7px;
    width: 10px; height: 2px;
    background-color: #0B57D0;
    border-radius: 1px;
}

/* ── Dialog overlay ──────────────────────────────────────── */
.dialog-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.dialog {
    background-color: #151515;
    color: #EDEDED;
    border-radius: 8px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.6);
    min-width: 1000px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
}

.dialog-body {
    flex: 1;
    overflow-y: auto;
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 18px;
}

.dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid #2A2A2A;
}

/* ── Group box ───────────────────────────────────────────── */
.group-box {
    border: 1px solid #333;
    border-radius: 6px;
    padding: 14px 14px 12px;
    position: relative;
}

.group-box-title {
    position: absolute;
    top: -10px;
    left: 10px;
    background-color: #151515;
    padding: 0 6px;
    color: #E1E1E1;
    font-weight: 600;
    font-size: 13px;
}

/* ── Grid helpers ────────────────────────────────────────── */
.grid-2col {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px 12px;
    align-items: center;
}

.grid-4col {
    display: grid;
    grid-template-columns: auto 1fr auto 1fr;
    gap: 8px 12px;
    align-items: center;
}

label {
    color: #E1E1E1;
    white-space: nowrap;
    font-size: 13px;
}

/* ── Inline dialog (add system prompt) ───────────────────── */
.mini-dialog-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0,0,0,0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
}

.mini-dialog {
    background-color: #1E1E1E;
    border: 1px solid #3A3A3A;
    border-radius: 6px;
    padding: 20px 24px;
    min-width: 340px;
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.mini-dialog h3 {
    color: #EDEDED;
    font-size: 15px;
    font-weight: 600;
}

.mini-dialog-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
}

/* ── File watcher table ──────────────────────────────────── */
.fw-table-wrapper {
    flex: 1;
    overflow: auto;
    background-color: #1A1A1A;
    border: 1px solid #333;
}

.fw-table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
    background-color: #1A1A1A;
    color: #F7F7F7;
}

.fw-table th {
    background-color: #212121;
    color: #E0E0E0;
    padding: 6px 8px;
    border: 1px solid #333;
    font-weight: 600;
    font-size: 13px;
    text-align: left;
    white-space: nowrap;
}

.fw-table td {
    padding: 4px 8px;
    border-bottom: 1px solid #2A2A2A;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.fw-table tr:hover td { background-color: #252525; }
.fw-table tr.selected td { background-color: #2A2A2A; }

.fw-table td.clickable { cursor: pointer; }

/* Column widths */
.fw-col-check    { width: 34px;  text-align: center; }
.fw-col-path     { width: auto;  }
.fw-col-status   { width: 120px; }
.fw-col-modified { width: 160px; }
.fw-col-action   { width: 80px;  text-align: center; }

/* Header checkbox cell */
.fw-table th.fw-col-check {
    text-align: center;
}

/* Row checkbox */
.fw-table td.fw-col-check {
    display: flex;
    align-items: center;
    justify-content: center;
}

/* ── Settings tables (without / except) ──────────────────── */
.settings-table-wrapper {
    min-height: 240px;
    overflow: auto;
    border: 1px solid #333;
    border-radius: 4px;
}

.settings-table {
    width: 100%;
    border-collapse: collapse;
    background-color: #1A1A1A;
    color: #F7F7F7;
}

.settings-table th {
    background-color: #212121;
    color: #E0E0E0;
    padding: 6px 8px;
    border: 1px solid #333;
    font-size: 13px;
    text-align: center;
}

.settings-table td {
    padding: 2px 4px;
    border-bottom: 1px solid #2A2A2A;
}

.settings-table td input[type="text"] {
    background: transparent;
    border: none;
    color: #F3F3F3;
    width: 100%;
    padding: 3px 4px;
    font-size: 13px;
}

.settings-table td input[type="text"]:focus {
    background-color: #252525;
    border-radius: 2px;
    outline: 1px solid #0B57D0;
}

.table-buttons {
    display: flex;
    gap: 8px;
    margin-top: 6px;
}

/* ── System row ──────────────────────────────────────────── */
.system-rows {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.system-row-buttons {
    display: flex;
    gap: 8px;
    margin-bottom: 4px;
}

/* ── Scrollbar styling (WebKit) ──────────────────────────── */
::-webkit-scrollbar { width: 8px; height: 8px; }
::-webkit-scrollbar-track { background: #1A1A1A; }
::-webkit-scrollbar-thumb { background: #3A3A3A; border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: #4A4A4A; }
"#;
