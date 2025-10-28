use std::time::Duration;

use chrono::{DateTime, Local};
use gpui::{
    App, Application, Bounds, Context, Entity, SharedString, Task, Timer, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::run_on_main;

#[derive(Clone)]
struct ClipboardEntry {
    content: String,
    timestamp: DateTime<Local>,
}

#[derive(Clone)]
struct History {
    entries: Vec<ClipboardEntry>,
}

impl History {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(20),
        }
    }

    fn add_entry(&mut self, content: String, cx: &mut Context<Self>) {
        if self.entries.len() >= 20 {
            self.entries.pop();
        }

        self.entries.insert(
            0,
            ClipboardEntry {
                content,
                timestamp: chrono::Local::now(),
            },
        );

        cx.notify();
    }
}

struct Clipboard {
    history: Entity<History>,
    monitor_task: Task<()>,
}

impl Clipboard {
    fn new(cx: &mut Context<Self>) -> Self {
        let history = cx.new(|_| History::new());

        let monitor_task = Self::spawn_monitor(history.clone(), cx);

        Self {
            history,
            monitor_task,
        }
    }

    fn spawn_monitor(history: Entity<History>, cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            let mut last_change_count = get_pasteboard_change_count();

            let POLL_INTERVAL = Duration::from_millis(100);

            loop {
                cx.background_executor().timer(POLL_INTERVAL).await;
                let current_count = get_pasteboard_change_count();

                if current_count != last_change_count {
                    last_change_count = current_count;
                    let content = match get_pasteboard_content() {
                        Some(content) => content,
                        None => continue,
                    };

                    cx.update_entity(&history, |history, cx| {
                        history.add_entry(content, cx);
                    })
                    .ok();
                }
            }
        })
    }
}

fn get_pasteboard_change_count() -> isize {
    run_on_main(|_mtm| unsafe { NSPasteboard::generalPasteboard().changeCount() })
}

fn get_pasteboard_content() -> Option<String> {
    run_on_main(|_mtm| unsafe {
        let pasteboard = NSPasteboard::generalPasteboard();
        match pasteboard.stringForType(NSPasteboardTypeString) {
            None => return None,
            Some(ns_string) => return Some(ns_string.to_string()),
        };
    })
}

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().size_8().bg(gpui::red()))
                    .child(div().size_8().bg(gpui::green()))
                    .child(div().size_8().bg(gpui::blue()))
                    .child(div().size_8().bg(gpui::yellow()))
                    .child(div().size_8().bg(gpui::black()))
                    .child(div().size_8().bg(gpui::white())),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld {
                    text: "World".into(),
                })
            },
        )
        .unwrap();
    });
}
