use std::{
    // ops::Add,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

// CPU Channel
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct CpuStats {
    load: f32,
}
foxglove::static_typed_channel!(pub(crate) CPU, "/cpu", CpuStats);

fn log_message() {
    // read the system data
    CPU.log(&CpuStats { load: 1.0 });
}

fn main() {
    let env = env_logger::Env::default().default_filter_or("debug");
    env_logger::init_from_env(env);

    let done = Arc::new(AtomicBool::default());
    ctrlc::set_handler({
        let done = done.clone();
        move || {
            done.store(true, Ordering::Relaxed);
        }
    })
    .expect("Failed to set SIGINT handler");

    foxglove::WebSocketServer::new()
        .start_blocking()
        .expect("Server failed to start");

    while !done.load(Ordering::Relaxed) {
        log_message();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
