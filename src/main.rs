use crate::winit::run_winit;

mod input_handler;
mod shell;
mod state;
mod winit;
fn main() {
    tracing_subscriber::fmt().init();
    run_winit();
}
