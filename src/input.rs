use smithay::{
    backend::input::{
        AbsolutePositionEvent, Axis, AxisSource, ButtonState, Event, InputBackend, InputEvent,
        KeyState, KeyboardKeyEvent, PointerAxisEvent, PointerButtonEvent,
    },
    input::{
        keyboard::{keysyms as xkb, FilterResult, Keysym, ModifiersState},
        pointer::{AxisFrame, ButtonEvent, MotionEvent},
    },
    reexports::wayland_server::{protocol::wl_surface::WlSurface, DisplayHandle},
    utils::SERIAL_COUNTER,
};

use crate::{
    state::{Backend, SplitState},
    FlyJa,
};

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
    ChangeWmState,
    ChangeSplitSate(SplitState),
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

impl<BackendData: Backend + 'static> FlyJa<BackendData> {
    pub fn process_input_event<I: InputBackend>(
        &mut self,
        _dh: &DisplayHandle,
        event: InputEvent<I>,
        _output_name: &str,
    ) {
        match event {
            InputEvent::Keyboard { event } => match self.keyboard_key_to_action::<I>(event) {
                KeyAction::Run(cmd) => {
                    if let Err(e) = std::process::Command::new(&cmd)
                        .env("WAYLAND_DISPLAY", self.socket_name.clone())
                        .spawn()
                    {
                        tracing::error!(cmd, err = %e, "Failed to start program");
                    }
                }
                KeyAction::ChangeWmState => {
                    self.wmstatus.status_change();
                }
                KeyAction::ChangeSplitSate(state) => {
                    self.set_split_state(state);
                }
                _ => {}
            },
            // Mouse or touch pad
            InputEvent::PointerButton { event } => {
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
                            Some(window.toplevel().wl_surface().clone()),
                            serial,
                        );
                        self.space.elements().for_each(|window| {
                            window.toplevel().send_configure();
                        });
                    } else {
                        self.space.elements().for_each(|window| {
                            window.set_activated(false);
                            window.toplevel().send_configure();
                        });
                        keyboard.set_focus(self, Option::<WlSurface>::None, serial);
                    }
                }
                pointer.button(
                    self,
                    &ButtonEvent {
                        button,
                        state: button_state,
                        serial,
                        time: event.time_msec(),
                    },
                );
            }
            InputEvent::PointerMotionAbsolute { event } => {
                let output = self.space.outputs().next().unwrap();

                let output_geo = self.space.output_geometry(output).unwrap();

                let pos = event.position_transformed(output_geo.size) + output_geo.loc.to_f64();
                let serial = SERIAL_COUNTER.next_serial();

                let pointer = self.seat.get_pointer().unwrap();

                let under = self.surface_under_pointer(&pointer);

                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: pos,
                        serial,
                        time: event.time_msec(),
                    },
                );
            }
            // scroll
            InputEvent::PointerAxis { event, .. } => {
                let source = event.source();

                let horizontal_amount = event.amount(Axis::Horizontal).unwrap_or_else(|| {
                    event.amount_discrete(Axis::Horizontal).unwrap_or(0.0) * 3.0
                });
                let vertical_amount = event
                    .amount(Axis::Vertical)
                    .unwrap_or_else(|| event.amount_discrete(Axis::Vertical).unwrap_or(0.0) * 3.0);

                let horizontal_amount_discrete = event.amount_discrete(Axis::Horizontal);

                let vertical_amount_discrete = event.amount_discrete(Axis::Vertical);

                let mut frame = AxisFrame::new(event.time_msec()).source(source);
                if horizontal_amount != 0.0 {
                    frame = frame.value(Axis::Horizontal, horizontal_amount);
                    if let Some(discrete) = horizontal_amount_discrete {
                        frame = frame.discrete(Axis::Horizontal, discrete as i32);
                    }
                } else if source == AxisSource::Finger {
                    frame = frame.stop(Axis::Horizontal);
                }
                if vertical_amount != 0.0 {
                    frame = frame.value(Axis::Vertical, vertical_amount);
                    if let Some(discrete) = vertical_amount_discrete {
                        frame = frame.discrete(Axis::Vertical, discrete as i32);
                    }
                } else if source == AxisSource::Finger {
                    frame = frame.stop(Axis::Vertical);
                }

                self.seat.get_pointer().unwrap().axis(self, frame);
            }
            _ => (),
        }
    }
}

impl<BackendData: Backend + 'static> FlyJa<BackendData> {
    fn keyboard_key_to_action<B: InputBackend>(&mut self, evt: B::KeyboardKeyEvent) -> KeyAction {
        let keycode = evt.key_code();
        let state = evt.state();
        tracing::debug!(keycode, ?state, "key");
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
    if modifiers.ctrl && modifiers.alt && keysym == xkb::KEY_BackSpace
        || modifiers.logo && keysym == xkb::KEY_q
    {
        // ctrl+alt+backspace = quit
        // logo + q = quit
        Some(KeyAction::Quit)
    } else if (xkb::KEY_XF86Switch_VT_1..=xkb::KEY_XF86Switch_VT_12).contains(&keysym) {
        // VTSwitch
        Some(KeyAction::VtSwitch(
            (keysym - xkb::KEY_XF86Switch_VT_1 + 1) as i32,
        ))
    } else if modifiers.logo && keysym == xkb::KEY_Return {
        // run terminal
        Some(KeyAction::Run("wezterm".into()))
    } else if modifiers.logo && (xkb::KEY_1..=xkb::KEY_9).contains(&keysym) {
        Some(KeyAction::Screen((keysym - xkb::KEY_1) as usize))
    } else if modifiers.logo && modifiers.shift && keysym == xkb::KEY_M {
        Some(KeyAction::ScaleDown)
    } else if modifiers.logo && keysym == xkb::KEY_P {
        Some(KeyAction::ChangeWmState)
    } else if modifiers.logo && modifiers.shift && keysym == xkb::KEY_P {
        Some(KeyAction::ScaleUp)
    } else if modifiers.logo && modifiers.shift && keysym == xkb::KEY_W {
        Some(KeyAction::TogglePreview)
    } else if modifiers.logo && modifiers.shift && keysym == xkb::KEY_R {
        Some(KeyAction::RotateOutput)
    } else if modifiers.logo && modifiers.shift && keysym == xkb::KEY_T {
        Some(KeyAction::ToggleTint)
    } else if modifiers.logo && keysym == xkb::KEY_v {
        Some(KeyAction::ChangeSplitSate(SplitState::V))
    } else if modifiers.logo && keysym == xkb::KEY_b {
        Some(KeyAction::ChangeSplitSate(SplitState::H))
    } else {
        None
    }
}
