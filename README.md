# Compare Observer

A multi-system file-watcher and deployment tool — rewritten in **Rust + Dioxus**.

## Stack

| Crate | Purpose |
|---|---|
| `dioxus` (desktop) | Reactive UI (replaces Qt Widgets) |
| `tokio` | Async runtime |
| `reqwest` | HTTP client (remote rule loading) |
| `serde` / `serde_json` | Config & API serialisation |
| `chrono` | Timestamps |
| `tracing` | Logging |

## Project layout

```
src/
├── main.rs                          # Entry point — dioxus::launch(App)
└── ui/
    ├── styles.rs                    # Global dark-theme CSS constant
    ├── dialogs/
    │   └── settings_dialog.rs       # Settings dialog (systems, Telegram, rules)
    └── widgets/
        └── file_watcher_table.rs    # Watched-files table with checkbox header
```

## Build

```powershell
cargo build --release
```

The binary is written to `target/release/compare-observer.exe`.

## Run (development)

```powershell
cargo run
```
