mod grab;
mod handle;
mod input;
mod shell;
mod state;
mod winit;
pub use state::FlyJa;

use smithay::reexports::wayland_server::Display;

use crate::winit::run_winit;

pub struct CalloopData {
    state: FlyJa,
    display: Display<FlyJa>,
}

static POSSIBLE_BACKENDS: &[&str] =
    &["--winit : Run flyja as a X11 or Wayland client using winit."];

fn main() {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    } else {
        tracing_subscriber::fmt().compact().init();
    }

    let arg = ::std::env::args().nth(1);
    match arg.as_ref().map(|s| &s[..]) {
        Some("--winit") => {
            tracing::info!("Starting flyja with winit backend");
            run_winit().unwrap();
        }
        Some(other) => {
            tracing::error!("Unknown backend: {}", other);
        }
        None => {
            println!("USAGE: flyja --backend");
            println!();
            println!("Possible backends are:");
            for backend in POSSIBLE_BACKENDS {
                println!("\t{}", backend);
            }
        }
    }
}
