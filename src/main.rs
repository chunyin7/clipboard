use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use std::sync::{Arc, Mutex};

use crate::{models::History, monitor::ClipboardMonitor};

mod models;
mod monitor;

fn main() {
    let history: History = Arc::new(Mutex::new(Vec::new()));
    let monitor = ClipboardMonitor::new(history);
    monitor.spawn();

    let manager = GlobalHotKeyManager::new().expect("Failed to create global hotkey manager");
    let hotkey = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyV);
    manager.register(hotkey).unwrap();
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.state == HotKeyState::Pressed {
            // TODO: perform ui stuff
        }
    }))
}
