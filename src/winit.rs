use std::time::Duration;

use flyja_logic::{Size, SizeAndPos};
use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        egl::EGLDevice,
        renderer::{
            ImportDma, ImportEgl, ImportMemWl, damage::OutputDamageTracker,
            element::surface::WaylandSurfaceRenderElement, gles::GlesRenderer,
        },
        winit::{self, WinitEvent, WinitGraphicsBackend},
    },
    delegate_dmabuf,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{calloop::EventLoop, wayland_server::Display},
    utils::{Rectangle, Transform},
    wayland::dmabuf::{
        DmabufFeedback, DmabufFeedbackBuilder, DmabufGlobal, DmabufHandler, DmabufState,
        ImportNotifier,
    },
};

use crate::state::{Backend, FlyjaState};

pub const OUTPUT_NAME: &str = "winit";

#[allow(unused)]
pub struct DmabufStateFly {
    state: DmabufState,
    global: DmabufGlobal,
    feedback: Option<DmabufFeedback>,
    full_redraw: u8,
}

pub struct WinitData {
    backend: WinitGraphicsBackend<GlesRenderer>,
    damager_tracker: OutputDamageTracker,
    dmabuf_state: DmabufStateFly,
}

impl Backend for WinitData {
    const HAS_GUSTURES: bool = true;
    const HAS_RELATIVE_MOTION: bool = true;
    fn seat_name(&self) -> String {
        "winit".to_owned()
    }
}

type FlyjaStateWinit = FlyjaState<WinitData>;

impl DmabufHandler for FlyjaState<WinitData> {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.backend_data.dmabuf_state.state
    }
    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        if self
            .backend_data
            .backend
            .renderer()
            .import_dmabuf(&dmabuf, None)
            .is_ok()
        {
            let _ = notifier.successful::<Self>();
        } else {
            notifier.failed();
        }
    }
}

delegate_dmabuf!(FlyjaState<WinitData>);

pub fn run_winit() {
    let mut event_loop: EventLoop<'_, FlyjaStateWinit> = EventLoop::try_new().unwrap();
    let display: Display<FlyjaStateWinit> = Display::new().unwrap();
    let (mut backend, winit) = winit::init::<GlesRenderer>().unwrap();

    let size = backend.window_size();

    let mode = Mode {
        size,
        refresh: 60_000,
    };

    let output = Output::new(
        OUTPUT_NAME.to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Smithay".into(),
            model: "winit".into(),
            serial_number: "Unknown".into(),
        },
    );

    let _global = output.create_global::<FlyjaState<WinitData>>(&display.handle());
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    let render_node = EGLDevice::device_for_display(backend.renderer().egl_context().display())
        .and_then(|device| device.try_get_render_node())
        .unwrap_or(None);
    let dmabuf_default_feedback = match render_node {
        Some(node) => {
            let dmabuf_formats = backend.renderer().dmabuf_formats();

            Some(
                DmabufFeedbackBuilder::new(node.dev_id(), dmabuf_formats)
                    .build()
                    .unwrap(),
            )
        }
        None => None,
    };

    let dmabuf_state = match dmabuf_default_feedback {
        Some(default_feedback) => {
            let mut dmabuf_state = DmabufState::new();
            let dmabuf_global = dmabuf_state
                .create_global_with_default_feedback::<FlyjaState<WinitData>>(
                    &display.handle(),
                    &default_feedback,
                );
            DmabufStateFly {
                state: dmabuf_state,
                global: dmabuf_global,
                feedback: Some(default_feedback),
                full_redraw: 0,
            }
        }
        None => {
            let dmabuf_formats = backend.renderer().dmabuf_formats();
            let mut dmabuf_state = DmabufState::new();
            let dmabuf_global = dmabuf_state
                .create_global::<FlyjaState<WinitData>>(&display.handle(), dmabuf_formats);
            DmabufStateFly {
                state: dmabuf_state,
                global: dmabuf_global,
                feedback: None,
                full_redraw: 0,
            }
        }
    };
    if backend
        .renderer()
        .bind_wl_display(&display.handle())
        .is_ok()
    {
        tracing::info!("EGL hardware-acceleration enabled");
    }

    let data = {
        let damager_tracker = OutputDamageTracker::from_output(&output);
        WinitData {
            backend,
            damager_tracker,
            dmabuf_state,
        }
    };

    let mut state = FlyjaState::init(display, &event_loop, data, true);
    state
        .shm_state
        .update_formats(state.backend_data.backend.renderer().shm_formats());
    state.space.map_output(&output, (0, 0));
    state.remap_space(SizeAndPos {
        size: Size {
            width: 0.,
            height: 0.,
        },
        position: flyja_logic::Position { x: 0., y: 0. },
    });

    event_loop
        .handle()
        .insert_source(winit, move |event, _, state| {
            match event {
                WinitEvent::Resized { size, .. } => {
                    output.change_current_state(
                        Some(Mode {
                            size,
                            refresh: 60_000,
                        }),
                        None,
                        None,
                        None,
                    );
                    state.remap_space(SizeAndPos {
                        size: Size {
                            width: size.w as f32,
                            height: size.h as f32,
                        },
                        position: flyja_logic::Position { x: 0., y: 0. },
                    });
                }
                WinitEvent::Input(event) => state.process_input_event(event),
                WinitEvent::Redraw => {
                    let backend = &mut state.backend_data.backend;
                    let size = backend.window_size();
                    let damage = Rectangle::from_size(size);

                    {
                        let (renderer, mut framebuffer) = backend.bind().unwrap();
                        smithay::desktop::space::render_output::<
                            _,
                            WaylandSurfaceRenderElement<GlesRenderer>,
                            _,
                            _,
                        >(
                            &output,
                            renderer,
                            &mut framebuffer,
                            1.0,
                            0,
                            [&state.space],
                            &[],
                            &mut state.backend_data.damager_tracker,
                            [0.1, 0.1, 0.1, 1.0],
                        )
                        .unwrap();
                    }
                    backend.submit(Some(&[damage])).unwrap();

                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        )
                    });

                    state.space.refresh();
                    state.popups.cleanup();
                    let _ = state.display_handle.flush_clients();

                    // Ask for redraw to schedule new frame.
                    backend.window().request_redraw();
                }
                WinitEvent::CloseRequested => {
                    state.signal.stop();
                }
                _ => (),
            }
        })
        .unwrap();
    if let Some(socket_name) = &state.socket_name {
        unsafe {
            std::env::set_var("WAYLAND_DISPLAY", socket_name);
        }
    }

    event_loop
        .run(None, &mut state, move |_| {
            // Smallvil is running
        })
        .unwrap();
}
