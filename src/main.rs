use crate::winit::run_winit;

mod grabs;
mod input_handler;
mod shell;
mod state;
mod winit;
mod udev;
fn main() {
    tracing_subscriber::fmt().init();
    run_winit();
}
