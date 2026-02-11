use std::collections::HashMap;

use smithay::{
    backend::{
        allocator::{dmabuf::Dmabuf, gbm::GbmAllocator},
        drm::{
            DrmDeviceFd, DrmNode, NodeType,
            exporter::gbm::GbmFramebufferExporter,
            output::{DrmOutput, DrmOutputManager},
        },
        egl::{EGLContext, context::ContextPriority},
        input::InputEvent,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            ImportDma,
            gles::GlesRenderer,
            multigpu::{GpuManager, gbm::GbmGlesBackend},
        },
        session::{Event as SessionEvent, Session, libseat::LibSeatSession},
        udev::{UdevBackend, all_gpus, primary_gpu},
    },
    delegate_dmabuf,
    desktop::utils::OutputPresentationFeedback,
    input::keyboard::LedState,
    output::Output,
    reexports::{
        calloop::{EventLoop, RegistrationToken},
        drm::control::{connector, crtc},
        input::{DeviceCapability, Libinput, LibinputInterface},
        wayland_server::{Display, DisplayHandle, backend::GlobalId, protocol::wl_surface},
    },
    utils::{Monotonic, Time},
    wayland::{
        dmabuf::{DmabufGlobal, DmabufHandler, DmabufState, ImportNotifier},
        drm_lease::{DrmLease, DrmLeaseState},
        drm_syncobj::DrmSyncobjState,
    },
};

use smithay_drm_extras::{
    display_info,
    drm_scanner::{DrmScanEvent, DrmScanner},
};

use crate::state::{Backend, FlyjaState};

struct DmabufStateFly {
    state: DmabufState,
    global: DmabufGlobal,
}

struct UdevOutputId {
    device_id: DrmNode,
    crtc: crtc::Handle,
}

pub struct UdevData {
    pub session: LibSeatSession,
    dh: DisplayHandle,
    dmabuf_sate: Option<DmabufStateFly>,
    syncobj_state: Option<DrmSyncobjState>,
    primary_gpu: DrmNode,
    gpus: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
    backends: HashMap<DrmNode, BackendData>,
    keyboards: Vec<smithay::reexports::input::Device>,
    // TODO: cursor and etc
}

struct SurfaceData {
    dh: DisplayHandle,
    device_id: DrmNode,
    render_node: Option<DrmNode>,
    output: Output,
    global: Option<GlobalId>,
    drm_output: DrmOutput<
        GbmAllocator<DrmDeviceFd>,
        GbmFramebufferExporter<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
    disable_direct_scanout: bool,

    //dmabuf_feedback: Option<SurfaceDmabufFeedback>,
    last_presentation_time: Option<Time<Monotonic>>,
    vblank_throttle_timer: Option<RegistrationToken>,
}

impl Drop for SurfaceData {
    fn drop(&mut self) {
        self.output.leave_all();
        if let Some(global) = self.global.take() {
            self.dh.remove_global::<FlyjaState<UdevData>>(global);
        }
    }
}

struct BackendData {
    surfaces: HashMap<crtc::Handle, SurfaceData>,
    non_desktop_connectors: Vec<(connector::Handle, crtc::Handle)>,
    leasing_global: Option<DrmLeaseState>,
    active_leases: Vec<DrmLease>,
    drm_output_manager: DrmOutputManager<
        GbmAllocator<DrmDeviceFd>,
        GbmFramebufferExporter<DrmDeviceFd>,
        Option<OutputPresentationFeedback>,
        DrmDeviceFd,
    >,
    drm_scanner: DrmScanner,
    render_node: Option<DrmNode>,
    registration_token: RegistrationToken,
}

impl DmabufHandler for FlyjaState<UdevData> {
    fn dmabuf_state(&mut self) -> &mut DmabufState {
        &mut self.backend_data.dmabuf_sate.as_mut().unwrap().state
    }
    fn dmabuf_imported(
        &mut self,
        _global: &DmabufGlobal,
        dmabuf: Dmabuf,
        notifier: ImportNotifier,
    ) {
        if self
            .backend_data
            .gpus
            .single_renderer(&self.backend_data.primary_gpu)
            .and_then(|mut renderer| renderer.import_dmabuf(&dmabuf, None))
            .is_ok()
        {
            dmabuf.set_node(self.backend_data.primary_gpu);
            let _ = notifier.successful::<Self>();
        } else {
            notifier.failed();
        }
    }
}

delegate_dmabuf!(FlyjaState<UdevData>);

impl Backend for UdevData {
    const HAS_RELATIVE_MOTION: bool = true;
    const HAS_GUSTURES: bool = true;

    fn seat_name(&self) -> String {
        self.session.seat()
    }

    fn reset_buffers(&mut self, output: &Output) {
        if let Some(id) = output.user_data().get::<UdevOutputId>() {
            if let Some(gpu) = self.backends.get_mut(&id.device_id) {
                if let Some(surface) = gpu.surfaces.get_mut(&id.crtc) {
                    surface.drm_output.reset_buffers();
                }
            }
        }
    }

    fn early_import(&mut self, surface: &wl_surface::WlSurface) {
        if let Err(err) = self.gpus.early_import(self.primary_gpu, surface) {
            tracing::warn!("Early buffer import failed: {}", err);
        }
    }

    fn update_led_state(&mut self, led_state: LedState) {
        for keyboard in self.keyboards.iter_mut() {
            keyboard.led_update(led_state.into());
        }
    }
}

pub fn run_udev() {
    // NOTE: initlialize base
    let mut event_loop: EventLoop<'_, FlyjaState<UdevData>> = EventLoop::try_new().unwrap();
    let display: Display<FlyjaState<UdevData>> = Display::new().unwrap();
    let mut display_handle = display.handle();

    /*
     * Initlize the session
     */
    let (session, notifier) = match LibSeatSession::new() {
        Ok(ret) => ret,
        Err(err) => {
            tracing::error!("Could not initlialize a session: {err}");
            return;
        }
    };

    /*
     * initlialize the compositer
     *
     * Different from the anvil, I just make a simple one
     */

    let primary_gpu = primary_gpu(session.seat())
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
                .expect("No GPU!")
        });

    tracing::info!("Using {} as primary gpu.", primary_gpu);

    let gpus = GpuManager::new(GbmGlesBackend::with_factory(|display| {
        let context = EGLContext::new_with_priority(display, ContextPriority::High)?;
        // TODO: maybe need an environment value
        let capabilities = unsafe { GlesRenderer::supported_capabilities(&context)? };
        Ok(unsafe { GlesRenderer::with_capabilities(context, capabilities)? })
    }))
    .unwrap();

    let data = UdevData {
        dh: display_handle.clone(),
        dmabuf_sate: None,
        syncobj_state: None,
        session,
        primary_gpu,
        gpus,
        backends: HashMap::new(),
        keyboards: Vec::new(),
    };

    let mut state = FlyjaState::init(display, &event_loop, data, true);

    let udev_backend = match UdevBackend::new(&state.seat_name) {
        Ok(ret) => ret,
        Err(err) => {
            tracing::error!(error = ?err, "Failed to initlialize udev backend");
            return;
        }
    };

    let mut libinput_context = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(
        state.backend_data.session.clone().into(),
    );

    /*
     * initlialize libinput backend
     */
    libinput_context.udev_assign_seat(&state.seat_name).unwrap();
    let libinput_backend = LibinputInputBackend::new(libinput_context.clone());

    /*
     * About the Device and input
     */
    event_loop
        .handle()
        .insert_source(libinput_backend, move |mut event, _, data| {
            //let dh = data.backend_data.dh.clone();
            if let InputEvent::DeviceAdded { device } = &mut event {
                if device.has_capability(DeviceCapability::Keyboard) {
                    if let Some(led_state) = data
                        .seat
                        .get_keyboard()
                        .map(|keyboard| keyboard.led_state())
                    {
                        device.led_update(led_state.into());
                    }
                    data.backend_data.keyboards.push(device.clone());
                }
            } else if let InputEvent::DeviceRemoved { ref device } = event {
                if device.has_capability(DeviceCapability::Keyboard) {
                    data.backend_data.keyboards.retain(|item| item != device);
                }
            }
            data.process_input_event(event);
        })
        .unwrap();

    event_loop
        .handle()
        .insert_source(notifier, move |event, &mut (), data| match event {
            SessionEvent::PauseSession => {
                libinput_context.suspend();
                tracing::info!("pausing session");
                for backend in data.backend_data.backends.values_mut() {
                    backend.drm_output_manager.pause();
                    backend.active_leases.clear();
                    if let Some(lease_global) = backend.leasing_global.as_mut() {
                        lease_global.suspend();
                    }
                }
            }
            SessionEvent::ActivateSession => {
                // NOTE: render
            }
        })
        .unwrap();
}
