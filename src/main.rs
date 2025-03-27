mod logger;

use clap::Parser;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use foxglove::McapWriter;
use logger::LoggerCollection;

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

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Mcap,
    Websocket,
    Both,
}

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
    /// Output format (mcap file, websocket server, or both)
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Both)]
    format: OutputFormat,
    /// Output path for mcap file
    #[arg(long, default_value = "output.mcap")]
    path: PathBuf,
    /// If set, overwrite an existing mcap file
    #[arg(short, long)]
    overwrite: bool,
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

    // Start websocket server if format is Websocket or Both
    if matches!(args.format, OutputFormat::Websocket | OutputFormat::Both) {
        foxglove::WebSocketServer::new()
            .start_blocking()
            .expect("Server failed to start");
    }

    // Initialize mcap writer if format is Mcap or Both
    let mcap = if matches!(args.format, OutputFormat::Mcap | OutputFormat::Both) {
        if args.overwrite && args.path.exists() {
            std::fs::remove_file(&args.path).expect("Failed to remove file");
        }
        Some(
            McapWriter::new()
                .create_new_buffered_file(&args.path)
                .expect("Failed to start mcap writer"),
        )
    } else {
        None
    };

    let mut logger_collection = LoggerCollection::new(&args);

    let mut elapsed_time_seconds: u64 = 0;
    while !done.load(Ordering::Relaxed)
        && args
            .timeout
            .map_or(true, |timeout| elapsed_time_seconds < timeout)
    {
        logger_collection.log_all();
        std::thread::sleep(std::time::Duration::from_millis(args.interval));
        elapsed_time_seconds += 1;
    }

    // Close mcap writer if it was initialized
    if let Some(writer) = mcap {
        writer.close().expect("Failed to close mcap writer");
    }
}
