use crate::models::{ClipboardEntry, History};
use chrono::Local;
use objc2_app_kit::{NSPasteboard, NSPasteboardTypeString};
use objc2_foundation::run_on_main;
use std::{thread, time::Duration};

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

pub struct ClipboardMonitor {
    history: History,
}

impl ClipboardMonitor {
    pub fn new(history: History) -> Self {
        Self {
            history: history.clone(),
        }
    }

    pub fn spawn(self) {
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
                    last_change_count = current_change_count;
                }
            }
        });
    }
}
