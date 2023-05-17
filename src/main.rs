fn main() {
    tracing_subscriber::fmt::init();
    pollster::block_on(rust_wgpu_lib::run());
}
