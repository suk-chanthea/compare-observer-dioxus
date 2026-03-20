/// Global application stylesheet — dark theme.
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
    background-color: #0D0D0D;
    color: #E5E5E5;
}

/* ── App shell ─────────────────────────────────────────── */
.app-root {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background-color: #0D0D0D;
}

/* ── Menu bar ───────────────────────────────────────────── */
.menu-bar {
    display: flex;
    align-items: center;
    background-color: #111111;
    border-bottom: 1px solid #232323;
    padding: 0 4px;
    height: 26px;
    flex-shrink: 0;
}

.menu-item {
    background: none;
    border: none;
    color: #D8D8D8;
    padding: 3px 14px;
    cursor: pointer;
    font-size: 13px;
    height: 100%;
}
.menu-item:hover { background-color: #2A2A2A; }

/* ── Toolbar ────────────────────────────────────────────── */
.toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 10px;
    background-color: #161616;
    border-bottom: 1px solid #232323;
    flex-shrink: 0;
    gap: 8px;
}

.toolbar-left {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 0;
}

.toolbar-right {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
}

.label-text {
    color: #C8C8C8;
    font-size: 13px;
    white-space: nowrap;
}

/* ── System selector tabs ───────────────────────────────── */
.sys-btn {
    background-color: #222222;
    color: #B0B0B0;
    border: 1px solid #3A3A3A;
    border-radius: 4px;
    padding: 4px 12px;
    font-size: 12px;
    cursor: pointer;
    white-space: nowrap;
}
.sys-btn:hover { background-color: #2E2E2E; }

.sys-btn-on {
    background-color: #17213A;
    border-color: #1A56DB;
    color: #E8E8E8;
}
.sys-btn-on:hover { background-color: #1C2848; }

/* ── Status dots ────────────────────────────────────────── */
.status-dot-label {
    color: #888888;
    font-size: 12px;
    white-space: nowrap;
}

/* ── Toolbar buttons ────────────────────────────────────── */
.btn {
    background-color: #2A2A2A;
    color: #FAFAFA;
    border: 1px solid #3A3A3A;
    border-radius: 4px;
    padding: 6px 12px;
    font-weight: 600;
    cursor: pointer;
    font-size: 13px;
    white-space: nowrap;
}
.btn:hover  { background-color: #3A3A3A; }
.btn:active { background-color: #4A4A4A; }

.btn-start {
    background-color: #1E3A5F;
    border-color: #1A56DB;
    color: #E8E8E8;
}
.btn-start:hover { background-color: #254878; }

.btn-stop {
    background-color: #7B1818;
    border-color: #C62828;
    color: #fff;
}
.btn-stop:hover { background-color: #922020; }

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

/* ── System action buttons (Copy / Copy Send / Assign To) ── */
.btn-action {
    color: #fff;
    border: none;
    border-radius: 4px;
    padding: 8px 0;
    cursor: pointer;
    font-size: 13px;
    font-weight: 600;
    width: 90px;
    text-align: center;
}

.btn-copy          { background-color: #1A56DB; }
.btn-copy:hover    { background-color: #1D4ED8; }

.btn-copy-send     { background-color: #1A7A2E; }
.btn-copy-send:hover { background-color: #1F9136; }

.btn-assign        { background-color: #6B21A8; }
.btn-assign:hover  { background-color: #7E22CE; }

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
    border-color: #1A56DB;
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
    background-color: #1A56DB;
    border-color: #1A56DB;
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

/* ── Main scrollable area ────────────────────────────────── */
.main-content {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    padding: 6px;
    gap: 8px;
}

/* ── System panel ────────────────────────────────────────── */
.system-panel {
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    height: 270px;
    border: 1px solid #252525;
    border-radius: 4px;
    background-color: #111111;
    overflow: hidden;
}

.system-desc-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 10px;
    border-bottom: 1px solid #252525;
    background-color: #141414;
    flex-shrink: 0;
}

.system-desc-label {
    color: #C0C0C0;
    font-size: 13px;
    white-space: nowrap;
}

.system-desc-input {
    flex: 1;
    background-color: #0F0F0F;
    color: #F3F3F3;
    border: 1px solid #2E2E2E;
    border-radius: 3px;
    padding: 4px 8px;
    font-size: 13px;
    outline: none;
}
.system-desc-input:focus { border-color: #1A56DB; }

.system-body {
    display: flex;
    flex: 1;
    overflow: hidden;
}

.system-table-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
}

.system-actions {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 7px;
    background-color: #131313;
    border-left: 1px solid #252525;
    flex-shrink: 0;
    justify-content: flex-start;
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

/* ── Mini dialog (add system) ────────────────────────────── */
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
    background-color: #111111;
    border: 1px solid #252525;
}

.fw-table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
    background-color: #111111;
    color: #E8E8E8;
}

.fw-table th {
    background-color: #181818;
    color: #D0D0D0;
    padding: 6px 8px;
    border-bottom: 1px solid #2A2A2A;
    font-weight: 600;
    font-size: 13px;
    text-align: left;
    white-space: nowrap;
    position: sticky;
    top: 0;
    z-index: 1;
}

.fw-table td {
    padding: 4px 8px;
    border-bottom: 1px solid #1E1E1E;
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.fw-table tr:hover td { background-color: #1A1A1A; }
.fw-table tr.selected td { background-color: #1C1C2A; }

.fw-table td.clickable { cursor: pointer; }

.fw-col-check    { width: 34px;  text-align: center; }
.fw-col-path     { width: auto;  }
.fw-col-status   { width: 120px; }
.fw-col-modified { width: 160px; }
.fw-col-action   { width: 80px;  text-align: center; }

.fw-table th.fw-col-check { text-align: center; }
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
    outline: 1px solid #1A56DB;
}

.table-buttons {
    display: flex;
    gap: 8px;
    margin-top: 6px;
}

/* ── System row (settings dialog) ────────────────────────── */
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
::-webkit-scrollbar-track { background: #111111; }
::-webkit-scrollbar-thumb { background: #333333; border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: #444444; }
"#;
