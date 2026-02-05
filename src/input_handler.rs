use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        KeyState, KeyboardKeyEvent, Keycode, PointerAxisEvent, PointerButtonEvent,
    },
    input::{
        keyboard::{FilterResult, KeyboardHandle, Keysym, ModifiersState, xkb::ModMask},
        pointer::{AxisFrame, ButtonEvent, MotionEvent},
    },
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::SERIAL_COUNTER,
    wayland::virtual_keyboard::VirtualKeyboardHandler,
};

use crate::state::{Backend, FlyjaState, TileState};

#[allow(unused)]
#[derive(Debug)]
enum KeyAction {
    Quit,
    Run(String),
    TiteStateChange(TileState),
    None,
}

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

impl<BackendData: Backend> FlyjaState<BackendData> {
    pub fn process_input_event<I: InputBackend>(&mut self, event: InputEvent<I>) {
        match event {
            InputEvent::Keyboard { event, .. } => match self.keyboard_key_to_action::<I>(event) {
                KeyAction::Run(cmd) => {
                    let mut process = std::process::Command::new(&cmd);
                    if let Some(socket_name) = &self.socket_name {
                        process.env("WAYLAND_DISPLAY", socket_name);
                    }
                    if let Err(e) = process.spawn() {
                        tracing::error!(cmd, err = %e, "Failed to start program");
                    }
                }
                KeyAction::TiteStateChange(tile) => {
                    self.tile_state = tile;
                }
                _ => {}
            },
            InputEvent::PointerMotion { .. } => {}
            InputEvent::PointerMotionAbsolute { event, .. } => {
                let output = self.space.outputs().next().unwrap();

                let output_geo = self.space.output_geometry(output).unwrap();

                let pos = event.position_transformed(output_geo.size) + output_geo.loc.to_f64();

                let serial = SERIAL_COUNTER.next_serial();

                let pointer = self.seat.get_pointer().unwrap();

                let under = self.surface_under(pos);

                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: pos,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerButton { event, .. } => {
                let pointer = self.seat.get_pointer().unwrap();
                let keyboard = self.seat.get_keyboard().unwrap();

                let serial = SERIAL_COUNTER.next_serial();

                let button = event.button_code();

                let button_state = event.state();

                if ButtonState::Pressed == button_state && !pointer.is_grabbed() {
                    if let Some((window, _loc)) = self
                        .space
                        .element_under(pointer.current_location())
                        .map(|(w, l)| (w.clone(), l))
                    {
                        self.space.raise_element(&window, true);
                        keyboard.set_focus(
                            self,
                            Some(window.toplevel().unwrap().wl_surface().clone()),
                            serial,
                        );
                        self.space.elements().for_each(|window| {
                            window.toplevel().unwrap().send_pending_configure();
                        });
                    } else {
                        self.space.elements().for_each(|window| {
                            window.set_activated(false);
                            window.toplevel().unwrap().send_pending_configure();
                        });
                        keyboard.set_focus(self, Option::<WlSurface>::None, serial);
                    }
                };

                pointer.button(
                    self,
                    &ButtonEvent {
                        button,
                        state: button_state,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerAxis { event, .. } => {
                let source = event.source();

                let horizontal_amount = event.amount(Axis::Horizontal).unwrap_or_else(|| {
                    event.amount_v120(Axis::Horizontal).unwrap_or(0.0) * 15.0 / 120.
                });
                let vertical_amount = event.amount(Axis::Vertical).unwrap_or_else(|| {
                    event.amount_v120(Axis::Vertical).unwrap_or(0.0) * 15.0 / 120.
                });
                let horizontal_amount_discrete = event.amount_v120(Axis::Horizontal);
                let vertical_amount_discrete = event.amount_v120(Axis::Vertical);

                let mut frame = AxisFrame::new(event.time_msec()).source(source);
                if horizontal_amount != 0.0 {
                    frame = frame.value(Axis::Horizontal, horizontal_amount);
                    if let Some(discrete) = horizontal_amount_discrete {
                        frame = frame.v120(Axis::Horizontal, discrete as i32);
                    }
                }
                if vertical_amount != 0.0 {
                    frame = frame.value(Axis::Vertical, vertical_amount);
                    if let Some(discrete) = vertical_amount_discrete {
                        frame = frame.v120(Axis::Vertical, discrete as i32);
                    }
                }

                if source == AxisSource::Finger {
                    if event.amount(Axis::Horizontal) == Some(0.0) {
                        frame = frame.stop(Axis::Horizontal);
                    }
                    if event.amount(Axis::Vertical) == Some(0.0) {
                        frame = frame.stop(Axis::Vertical);
                    }
                }

                let pointer = self.seat.get_pointer().unwrap();
                pointer.axis(self, frame);
                pointer.frame(self);
            }
            _ => {}
        }
    }
}
impl<BackendData: Backend + 'static> FlyjaState<BackendData> {
    fn keyboard_key_to_action<B: InputBackend>(&mut self, evt: B::KeyboardKeyEvent) -> KeyAction {
        let keycode = evt.key_code();
        let state = evt.state();
        let serial = SERIAL_COUNTER.next_serial();
        let time = Event::time_msec(&evt);
        let keyboard = self.seat.get_keyboard().unwrap();
        keyboard
            .input(
                self,
                keycode,
                state,
                serial,
                time,
                |_, modifiers, handle| {
                    let keysym = handle.modified_sym();
                    if let KeyState::Pressed = state {
                        let action = process_keyboard_shortcut(*modifiers, keysym);
                        action
                            .map(FilterResult::Intercept)
                            .unwrap_or(FilterResult::Forward)
                    } else {
                        FilterResult::Forward
                    }
                },
            )
            .unwrap_or(KeyAction::None)
    }
}
fn process_keyboard_shortcut(modifiers: ModifiersState, keysym: Keysym) -> Option<KeyAction> {
    if modifiers.logo && keysym == Keysym::Return {
        // run terminal
        Some(KeyAction::Run("wezterm".into()))
    } else if modifiers.logo && keysym == Keysym::v {
        Some(KeyAction::TiteStateChange(TileState::Vertical))
    } else if modifiers.logo && keysym == Keysym::b {
        Some(KeyAction::TiteStateChange(TileState::Horizontal))
    } else {
        None
    }
}
