use smithay::{
    backend::input::{InputBackend,Event, InputEvent, KeyboardKeyEvent},
    reexports::wayland_server::DisplayHandle, utils::SERIAL_COUNTER,
};

use crate::FlyJa;

/// Possible results of a keyboard action
#[allow(dead_code)]
#[derive(Debug)]
enum KeyAction {
    /// Quit the compositor
    Quit,
    /// Trigger a vt-switch
    VtSwitch(i32),
    /// run a command
    Run(String),
    /// Switch the current screen
    Screen(usize),
    ScaleUp,
    ScaleDown,
    TogglePreview,
    RotateOutput,
    ToggleTint,
    /// Do nothing more
    None,
}

impl FlyJa {
    pub fn process_input_event<I: InputBackend>(
        &mut self,
        dh: &DisplayHandle,
        event: InputEvent<I>,
        output_name: &str,
    ) {
        match event {
            InputEvent::Keyboard { event } => {}
            InputEvent::PointerMotionAbsolute { event } => {}
            InputEvent::PointerButton { event } => {}
            InputEvent::PointerAxis { event } => {}
            _ => (),
        }
    }
}

impl FlyJa {
    fn keyboard_key_to_action<B: InputBackend>(&mut self, evt: B::KeyboardKeyEvent) -> KeyAction {
        let keycode = evt.key_code();
        let state = evt.state();
        tracing::debug!(keycode, ?state, "key");
        let serial = SERIAL_COUNTER.next_serial();
        let time = Event::time_msec(&evt);
        todo!()
    }
}
