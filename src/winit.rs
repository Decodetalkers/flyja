use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        renderer::{ImportDma, damage::OutputDamageTracker, gles::GlesRenderer},
        winit::WinitGraphicsBackend,
    },
    delegate_dmabuf,
    reexports::wayland_server::Display,
    wayland::{
        compositor::CompositorState,
        dmabuf::{DmabufFeedback, DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
    },
};

use crate::state::{Backend, FlyjaState};

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
    //let mut display = Display::new().unwrap();
    //let dh = display.handle();
    //let compositer_state = CompositorState::new::<FlyjaStateWinit>(&dh);
}
