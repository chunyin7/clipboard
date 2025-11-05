use dispatch2::run_on_main;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use objc2_app_kit::NSApplication;
use std::sync::{Arc, Mutex};

use crate::{models::History, monitor::ClipboardMonitor, panel::init_panel};

mod models;
mod monitor;
mod panel;

fn main() {
    let history: History = Arc::new(Mutex::new(Vec::new()));
    let panel_bound = Arc::new(init_panel(history.clone()));

    let panel_for_monitor = panel_bound.clone();
    let monitor = ClipboardMonitor::new(history.clone());
    monitor.spawn(move || {
        panel_for_monitor.get_on_main(|panel| panel.refresh_history());
    });

    let manager = Box::leak(Box::new(
        GlobalHotKeyManager::new().expect("Failed to create global hotkey manager"),
    ));
    let hotkey = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyV);
    manager.register(hotkey).unwrap();

    let panel_for_hotkey = panel_bound.clone();
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.state == HotKeyState::Pressed {
            panel_for_hotkey.get_on_main(|panel| panel.toggle());
        }
    }));

    // ensure table starts with current history snapshot
    panel_bound.get_on_main(|panel| panel.refresh_history());

    run_on_main(|mtm| {
        let app = unsafe { NSApplication::sharedApplication(mtm) };
        unsafe { app.run() };
    });
}
