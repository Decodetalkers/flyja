use smithay::{
    backend::input::{KeyState, Keycode},
    input::keyboard::{FilterResult, KeyboardHandle, xkb::ModMask},
    utils::SERIAL_COUNTER,
    wayland::virtual_keyboard::VirtualKeyboardHandler,
};

use crate::state::{Backend, FlyjaState};

impl<BackendData: Backend> VirtualKeyboardHandler for FlyjaState<BackendData> {
    fn on_keyboard_event(
        &mut self,
        keycode: Keycode,
        state: KeyState,
        time: u32,
        keyboard: KeyboardHandle<Self>,
    ) {
        let serial = SERIAL_COUNTER.next_serial();
        keyboard.input(self, keycode, state, serial, time, |_, _, _| {
            FilterResult::Forward::<bool>
        });
    }
    fn on_keyboard_modifiers(
        &mut self,
        _depressed_mods: ModMask,
        _latched_mods: ModMask,
        _locked_mods: ModMask,
        _keyboard: KeyboardHandle<Self>,
    ) {
    }
}
