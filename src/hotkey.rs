use std::thread;

use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};
use std::cell::OnceCell;

pub struct HotkeyService {
    manager: OnceCell<GlobalHotKeyManager>,
}

impl HotkeyService {
    pub fn new() -> Self {
        let manager = GlobalHotKeyManager::new().expect("Failed to create global hotkey manager");
        let hotkey = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyV);
        manager.register(hotkey);

        let cell = OnceCell::new();
        cell.set(manager);

        Self { manager: cell }
    }

    pub fn spawn(&self) {
        let rx = GlobalHotKeyEvent::receiver();
        thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                if event.state == HotKeyState::Pressed {
                    // TODO: notify ui
                }
            }
        });
    }
}
