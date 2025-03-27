mod logger;

use clap::Parser;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use foxglove::McapWriter;
use logger::LoggerCollection;

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
