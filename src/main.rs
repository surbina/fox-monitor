use clap::Parser;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use sysinfo::{Components, Disks, Networks, ProcessesToUpdate, System};

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

// Processes
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct ProcessStats {
    pid: u32,
    parent_pid: String,
    name: String,
    status: String,
    cpu_usage: f32,
    memory_usage_kb: u64,
    start_time_seconds: u64,
    run_time_seconds: u64,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct ProcessesStats {
    processes: Vec<ProcessStats>,
}
foxglove::static_typed_channel!(pub(crate) PROCESSES, "/processes", ProcessesStats);

fn log_processes_info(sys: &mut System) {
    sys.refresh_processes(ProcessesToUpdate::All, true);

    PROCESSES.log(&ProcessesStats {
        processes: sys
            .processes()
            .iter()
            .map(|(pid, process)| ProcessStats {
                pid: pid.as_u32(),
                parent_pid: match process.parent() {
                    Some(parent) => parent.as_u32().to_string(),
                    None => "Unknown".to_string(),
                },
                name: process.name().to_string_lossy().to_string(),
                status: process.status().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage_kb: process.memory() / 1024,
                start_time_seconds: process.start_time(),
                run_time_seconds: process.run_time(),
            })
            .collect(),
    });
}

// System
#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
struct SystemStats {
    name: String,
    kernel_version: String,
    os_version: String,
    os_long_version: String,
    host_name: String,
    kernel: String,
    boot_time_seconds: u64,
    uptime_seconds: u64,
    load_avg_one: f64,
    load_avg_five: f64,
    load_avg_fifteen: f64,
}
foxglove::static_typed_channel!(pub(crate) SYSTEM, "/system", SystemStats);

fn log_system_info() {
    SYSTEM.log(&SystemStats {
        name: System::name().unwrap_or_else(|| "<unknown>".to_owned()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "<unknown>".to_owned()),
        os_version: System::os_version().unwrap_or_else(|| "<unknown>".to_owned()),
        os_long_version: System::long_os_version().unwrap_or_else(|| "<unknown>".to_owned()),
        host_name: System::host_name().unwrap_or_else(|| "<unknown>".to_owned()),
        kernel: System::kernel_version().unwrap_or_else(|| "<unknown>".to_owned()),
        boot_time_seconds: System::boot_time(),
        uptime_seconds: System::uptime(),
        load_avg_one: System::load_average().one,
        load_avg_five: System::load_average().five,
        load_avg_fifteen: System::load_average().fifteen,
    });
}

// TODO
// - Add flag to control logging format (webserver, mcap file, both)
// - Add flag to config path (mcap file)

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Log cpu info
    #[arg(short, long)]
    cpu: bool,
    /// Log memory info
    #[arg(short, long)]
    memory: bool,
    /// Log components temperature
    #[arg(short, long)]
    temperature: bool,
    /// Log disks info
    #[arg(short, long)]
    disks: bool,
    /// Log networks info
    #[arg(short, long)]
    networks: bool,
    /// Log processes info
    #[arg(short, long)]
    processes: bool,
    /// Log system info
    #[arg(short, long)]
    system: bool,
    /// Interval between logs in seconds
    #[arg(short, long, default_value_t = 1000)]
    interval: u64,
    /// If provided, the program will exit after the timeout (in seconds)
    #[arg(long)]
    timeout: Option<u64>,
}

fn main() {
    let env = env_logger::Env::default().default_filter_or("debug");
    env_logger::init_from_env(env);

    let args = Cli::parse();

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

    let mut system = if args.cpu || args.memory || args.processes {
        Some(System::new_all())
    } else {
        None
    };

    let mut components = if args.temperature {
        Some(Components::new_with_refreshed_list())
    } else {
        None
    };

    let mut disks = if args.disks {
        Some(Disks::new_with_refreshed_list())
    } else {
        None
    };

    let mut networks = if args.networks {
        Some(Networks::new_with_refreshed_list())
    } else {
        None
    };

    if let Some(sys) = system.as_mut() {
        sys.refresh_all();
    }

    let mut elapsed_time_seconds: u64 = 0;
    while !done.load(Ordering::Relaxed)
        && args
            .timeout
            .map_or(true, |timeout| elapsed_time_seconds < timeout)
    {
        if args.cpu || args.memory || args.processes {
            if let Some(sys) = system.as_mut() {
                if args.cpu {
                    log_cpu_info(sys);
                }
                if args.memory {
                    log_memory_info(sys);
                }
                if args.processes {
                    log_processes_info(sys);
                }
            }
        }
        if args.temperature {
            if let Some(comps) = components.as_mut() {
                log_components_info(comps);
            }
        }
        if args.disks {
            if let Some(d) = disks.as_mut() {
                log_disks_info(d);
            }
        }
        if args.networks {
            if let Some(nets) = networks.as_mut() {
                log_networks_info(nets);
            }
        }
        if args.system {
            log_system_info();
        }
        std::thread::sleep(std::time::Duration::from_millis(args.interval));
        elapsed_time_seconds = elapsed_time_seconds + 1;
    }
}
