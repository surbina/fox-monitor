use std::{
    // ops::Add,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

// create a channel for my own data
foxglove::static_typed_channel!(pub(crate) CPU_LOAD, "/cpu_load", foxglove::schemas::Vector3);

fn log_message() {
    // read the system data
    // log the data to the channel

    CPU_LOAD.log(&foxglove::schemas::Vector3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    });
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
        std::thread::sleep(std::time::Duration::from_millis(33));
    }
}
