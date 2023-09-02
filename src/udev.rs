use smithay::{
    backend::{
        drm::{DrmNode, NodeType},
        session::{libseat::LibSeatSession, Session},
        udev::{all_gpus, primary_gpu, UdevBackend},
    },
    reexports::calloop::EventLoop,
    reexports::wayland_server::Display,
};

use crate::{state::Backend, FlyJa};
use tracing::{error, info};
pub struct UdevData {
    pub session: LibSeatSession,
}

impl Backend for UdevData {
    const HAS_RELATIVE_MOTION: bool = true;
    fn seat_name(&self) -> String {
        self.session.seat()
    }
}

pub fn run_udev() {
    let mut event_loop = EventLoop::try_new().unwrap();
    let mut display = Display::new().unwrap();

    let (session, notifier) = match LibSeatSession::new() {
        Ok(ret) => ret,
        Err(err) => {
            error!("Could not initialize a session: {}", err);
            return;
        }
    };
    let primary_gpu = primary_gpu(&session.seat())
        .unwrap()
        .and_then(|x| {
            DrmNode::from_path(x)
                .ok()?
                .node_with_type(NodeType::Render)?
                .ok()
        })
        .unwrap_or_else(|| {
            all_gpus(session.seat())
                .unwrap()
                .into_iter()
                .find_map(|x| DrmNode::from_path(x).ok())
                .expect("No Gpu!")
        });

    info!("Using {} as primary gpu.", primary_gpu);

    let data = UdevData { session };

    let mut state = FlyJa::init(data, &mut event_loop, &mut display);

    let udev_backend = match UdevBackend::new(&state.seat_name) {
        Ok(ret) => ret,
        Err(err) => {
            error!(error = ?err,"Failed to initialize udev backend");
            return;
        }
    };
}
