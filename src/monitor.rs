use crate::models::{ClipboardEntry, History};
use chrono::Local;
use dispatch2::run_on_main;
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use std::{thread, time::Duration};

fn get_pasteboard_change_count() -> isize {
    run_on_main(|_mtm| NSPasteboard::generalPasteboard().changeCount())
}

fn get_pasteboard_content() -> Option<String> {
    run_on_main(|_mtm| unsafe {
        NSPasteboard::generalPasteboard()
            .stringForType(NSPasteboardTypeString)
            .map(|ns_string| ns_string.to_string())
    })
}

pub struct ClipboardMonitor {
    history: History,
}

impl ClipboardMonitor {
    pub fn new(history: History) -> Self {
        Self { history }
    }

    pub fn spawn<F>(self, mut on_change: F)
    where
        F: FnMut() + Send + 'static,
    {
        thread::spawn(move || {
            let mut last_change_count = get_pasteboard_change_count();

            loop {
                thread::sleep(Duration::from_millis(100));
                let current_change_count = get_pasteboard_change_count();
                if current_change_count != last_change_count {
                    if let Some(content) = get_pasteboard_content() {
                        let mut history = self.history.lock().unwrap();
                        history.insert(
                            0,
                            ClipboardEntry {
                                content,
                                timestamp: Local::now(),
                            },
                        );
                        if history.len() > 20 {
                            history.truncate(20);
                        }
                    }
                    on_change();
                    last_change_count = current_change_count;
                }
            }
        });
    }
}
