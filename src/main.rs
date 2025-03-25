use sysinfo::{Components, Disks, Networks, System};

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

// CPU Channel
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct CoreStats {
    usage: f32,
    frequency_mhz: u64,
    vendor_id: String,
    brand: String,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct CpuStats {
    usage: f32,
    physical_cores: u16,
    cores: Vec<CoreStats>,
}
foxglove::static_typed_channel!(pub(crate) CPU, "/cpu", CpuStats);

fn log_cpu_info(sys: &mut System) {
    sys.refresh_cpu_all();

    CPU.log(&CpuStats {
        usage: sys.global_cpu_usage(),
        physical_cores: System::physical_core_count(&sys)
            .map(|c| c.to_string())
            .unwrap_or_else(|| "Unknown".to_owned())
            .trim()
            .parse()
            .unwrap_or(0),
        cores: sys
            .cpus()
            .iter()
            .map(|c| CoreStats {
                usage: c.cpu_usage(),
                frequency_mhz: c.frequency(),
                vendor_id: c.vendor_id().to_string(),
                brand: c.brand().to_string(),
            })
            .collect(),
    });
}

// Memory Channel
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct MemoryStats {
    total_kb: u64,
    available_kb: u64,
    used_kb: u64,
    swap_total_kb: u64,
    swap_used_kb: u64,
}
foxglove::static_typed_channel!(pub(crate) MEMORY, "/memory", MemoryStats);

fn log_memory_info(sys: &mut System) {
    sys.refresh_memory();

    MEMORY.log(&MemoryStats {
        total_kb: sys.total_memory(),
        available_kb: sys.available_memory(),
        used_kb: sys.used_memory(),
        swap_total_kb: sys.total_swap(),
        swap_used_kb: sys.used_swap(),
    });
}

// Components Channel
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct ComponentStats {
    label: String,
    temperature: f32,
}

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

// Disks channel
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct DiskStats {
    name: String,
    mount_point: String,
    total_read_kb: u64,
    total_written_kb: u64,
    read_kb: u64,
    written_kb: u64,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct DisksStats {
    disks: Vec<DiskStats>,
}
foxglove::static_typed_channel!(pub(crate) DISKS, "/disks", DisksStats);

fn log_disks_info(disks: &mut Disks) {
    disks.refresh(true);

    DISKS.log(&DisksStats {
        disks: disks
            .iter()
            .map(|d| DiskStats {
                name: d.name().to_str().unwrap_or("Unknown").to_string(),
                mount_point: d.mount_point().to_str().unwrap_or("Unknown").to_string(),
                total_read_kb: d.usage().total_read_bytes / 1024,
                total_written_kb: d.usage().total_written_bytes / 1024,
                read_kb: d.usage().read_bytes / 1024,
                written_kb: d.usage().written_bytes / 1024,
            })
            .collect(),
    });
}

// Networks channel
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct NetworkStats {
    interface_name: String,
    mac_address: String,
    received: u64,
    transmitted: u64,
    total_received: u64,
    total_transmitted: u64,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct NetworksStats {
    networks: Vec<NetworkStats>,
}
foxglove::static_typed_channel!(pub(crate) NETWORKS, "/networks", NetworksStats);

fn log_networks_info(networks: &mut Networks) {
    networks.refresh(true);

    NETWORKS.log(&NetworksStats {
        networks: networks
            .iter()
            .map(|(interface_name, data)| NetworkStats {
                interface_name: interface_name.to_string(),
                mac_address: data.mac_address().to_string(),
                received: data.received(),
                transmitted: data.transmitted(),
                total_received: data.total_received(),
                total_transmitted: data.total_transmitted(),
            })
            .collect(),
    });
}

// Users: TODO

// Processes: TODO

// System: TODO

// TODO
// - Add flags to enable/disable logging for each channel
// - Add flag to set the log interval
// - Add flag to control logging format (webserver, mcap file, both)
// - Add flag to set timer (automatically quit after X time of running)

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
    let mut disks = Disks::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();
    system.refresh_all();

    while !done.load(Ordering::Relaxed) {
        log_cpu_info(&mut system);
        log_memory_info(&mut system);
        log_components_info(&mut components);
        log_disks_info(&mut disks);
        log_networks_info(&mut networks);
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
