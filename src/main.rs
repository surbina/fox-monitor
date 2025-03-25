use sysinfo::{Components, System};

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

// CPU Channel
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct CpuStats {
    usage: f32,
    physical_cores: u16,
    core_usage: Vec<f32>,
}
foxglove::static_typed_channel!(pub(crate) CPU, "/cpu", CpuStats);

fn log_cpu_info(sys: &mut System) {
    sys.refresh_cpu_all();

    // read the system data
    CPU.log(&CpuStats {
        usage: sys.global_cpu_usage(),
        physical_cores: System::physical_core_count(&sys)
            .map(|c| c.to_string())
            .unwrap_or_else(|| "Unknown".to_owned())
            .trim()
            .parse()
            .unwrap_or(0),
        core_usage: sys.cpus().iter().map(|c| c.cpu_usage()).collect(),
    });
}

// Memory Channel
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct MemoryStats {
    total: u64,
    available: u64,
    used: u64,
    swap_total: u64,
    swap_used: u64,
}
foxglove::static_typed_channel!(pub(crate) MEMORY, "/memory", MemoryStats);

fn log_memory_info(sys: &mut System) {
    sys.refresh_memory();

    MEMORY.log(&MemoryStats {
        total: sys.total_memory(),
        available: sys.available_memory(),
        used: sys.used_memory(),
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
    });
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct ComponentStats {
    label: String,
    temperature: f32,
}

// Components Channel
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct ComponentsStats {
    components: Vec<ComponentStats>,
}
foxglove::static_typed_channel!(pub(crate) COMPONENTS, "/components", ComponentsStats);

fn log_components_info(components: &mut Components) {
    components.refresh(true);

    COMPONENTS.log(&ComponentsStats {
        components: components
            .iter()
            .map(|c| ComponentStats {
                label: c.label().to_string(),
                temperature: c.temperature().unwrap_or(0.0),
            })
            .collect(),
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

    let mut system = System::new_all();
    let mut components = Components::new_with_refreshed_list();
    system.refresh_all();

    while !done.load(Ordering::Relaxed) {
        log_cpu_info(&mut system);
        log_memory_info(&mut system);
        log_components_info(&mut components);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
