use crate::state::Backend;
use crate::CalloopData;
use crate::FlyJa;
use smithay::{
    backend::{
        renderer::{
            damage::OutputDamageTracker, element::surface::WaylandSurfaceRenderElement,
            gles::GlesRenderer,
        },
        winit::{self, WinitError, WinitEvent, WinitEventLoop, WinitGraphicsBackend},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{
            timer::{TimeoutAction, Timer},
            EventLoop,
        },
        wayland_server::Display,
    },
    utils::{Rectangle, Transform},
};
use std::time::Duration;

pub const OUTPUT_NAME: &str = "winit";

pub struct WinitData;

impl Backend for WinitData {
    fn seat_name(&self) -> String {
        "Winit".to_string()
    }
}

pub fn run_winit() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop: EventLoop<CalloopData<WinitData>> = EventLoop::try_new()?;

    let mut display: Display<FlyJa<WinitData>> = Display::new()?;
    let data = WinitData;
    let state = FlyJa::init(data, &mut event_loop, &mut display);

    let mut data = CalloopData { state, display };
    init_winit(&mut event_loop, &mut data)?;

    event_loop.run(None, &mut data, move |_| {})?;
    Ok(())
}

fn init_winit<T>(
    event_loop: &mut EventLoop<CalloopData<T>>,
    data: &mut CalloopData<T>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Backend + 'static,
{
    let display = &mut data.display;
    let state = &mut data.state;

    let (mut backend, mut winit) = winit::init()?;

    let mode = Mode {
        size: backend.window_size().physical_size,
        refresh: 60_000,
    };

    let output = Output::new(
        "winit".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Flyja".into(),
            model: "Winit".into(),
        },
    );

    let _global = output.create_global::<FlyJa<T>>(&display.handle());
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    state.space.map_output(&output, (0, 0));

    let mut damage_tracked_renderer = OutputDamageTracker::from_output(&output);

    std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);

    let mut full_redraw = 0u8;

    let timer = Timer::immediate();
    event_loop
        .handle()
        .insert_source(timer, move |_, _, data| {
            winit_dispatch(
                &mut backend,
                &mut winit,
                data,
                &output,
                &mut damage_tracked_renderer,
                &mut full_redraw,
            )
            .unwrap();
            TimeoutAction::ToDuration(Duration::from_millis(16))
        })?;

    Ok(())
}

fn winit_dispatch<T>(
    backend: &mut WinitGraphicsBackend<GlesRenderer>,
    winit: &mut WinitEventLoop,
    data: &mut CalloopData<T>,
    output: &Output,
    damage_tracked_renderer: &mut OutputDamageTracker,
    full_redraw: &mut u8,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Backend + 'static,
{
    let display = &mut data.display;
    let state = &mut data.state;

    let res = winit.dispatch_new_events(|event| match event {
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
        }
        WinitEvent::Input(event) => {
            state.process_input_event(&display.handle(), event, OUTPUT_NAME)
        } //state.process_input_event(event),
        _ => (),
    });

    if let Err(WinitError::WindowClosed) = res {
        // Stop the loop
        state.loop_signal.stop();

        return Ok(());
    } else {
        res?;
    }

    *full_redraw = full_redraw.saturating_sub(1);

    let size = backend.window_size().physical_size;
    let damage = Rectangle::from_loc_and_size((0, 0), size);

    backend.bind()?;
    smithay::desktop::space::render_output::<_, WaylandSurfaceRenderElement<GlesRenderer>, _, _>(
        output,
        backend.renderer(),
        1.0,
        0,
        [&state.space],
        &[],
        damage_tracked_renderer,
        [0.1, 0.1, 0.1, 1.0],
    )?;
    backend.submit(Some(&[damage]))?;

    state.space.elements().for_each(|window| {
        window.send_frame(
            output,
            state.start_time.elapsed(),
            Some(Duration::ZERO),
            |_, _| Some(output.clone()),
        )
    });

    state.space.refresh();
    state.popups.cleanup();
    display.flush_clients()?;

    Ok(())
}
