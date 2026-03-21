//! Log dialog — Dioxus port of `log_dialog.cpp`.
//! Shows a timestamped list of application events.

use dioxus::prelude::*;

#[component]
pub fn LogDialog(
    entries: Signal<Vec<(String, String)>>,
    on_close: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "dialog log-dialog",
                // Stop click-through to overlay
                onclick: move |e| e.stop_propagation(),

                // Header
                div {
                    class: "log-header",
                    span { class: "log-title", "Application Logs" }
                    button {
                        class: "log-close-btn",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                // Table
                div {
                    class: "log-table-wrapper",
                    table {
                        class: "log-table",
                        thead {
                            tr {
                                th { style: "width:170px;", "Timestamp" }
                                th { "Message" }
                            }
                        }
                        tbody {
                            for (ts, msg) in entries.read().iter().rev() {
                                tr {
                                    td { class: "log-ts", "{ts}" }
                                    td { "{msg}" }
                                }
                            }
                        }
                    }
                }

                // Footer buttons
                div {
                    class: "dialog-footer",
                    button {
                        class: "btn",
                        onclick: move |_| entries.write().clear(),
                        "Clear"
                    }
                    button {
                        class: "btn",
                        onclick: move |_| on_close.call(()),
                        "Close"
                    }
                }
            }
        }
    }
}
