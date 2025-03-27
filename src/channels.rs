// This file contains the channels used to log the system information.

use schemars::JsonSchema;
use serde::Serialize;

// CPU Channel
#[derive(Debug, Serialize, JsonSchema)]
pub struct CoreStats {
    pub usage: f32,
    pub frequency_mhz: u64,
    pub vendor_id: String,
    pub brand: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CpuStats {
    pub usage: f32,
    pub physical_cores: u16,
    pub cores: Vec<CoreStats>,
}
foxglove::static_typed_channel!(pub(crate) CPU, "/cpu", CpuStats);

// Memory Channel
#[derive(Debug, Serialize, JsonSchema)]
pub struct MemoryStats {
    pub total_kb: u64,
    pub available_kb: u64,
    pub used_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
}
foxglove::static_typed_channel!(pub(crate) MEMORY, "/memory", MemoryStats);

// Components Channel
#[derive(Debug, Serialize, JsonSchema)]
pub struct ComponentStats {
    pub label: String,
    pub temperature: f32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ComponentsStats {
    pub components: Vec<ComponentStats>,
}
foxglove::static_typed_channel!(pub(crate) COMPONENTS, "/components", ComponentsStats);

// Disks channel
#[derive(Debug, Serialize, JsonSchema)]
pub struct DiskStats {
    pub name: String,
    pub mount_point: String,
    pub total_read_kb: u64,
    pub total_written_kb: u64,
    pub read_kb: u64,
    pub written_kb: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DisksStats {
    pub disks: Vec<DiskStats>,
}
foxglove::static_typed_channel!(pub(crate) DISKS, "/disks", DisksStats);

// Networks channel
#[derive(Debug, Serialize, JsonSchema)]
pub struct NetworkStats {
    pub interface_name: String,
    pub mac_address: String,
    pub received: u64,
    pub transmitted: u64,
    pub total_received: u64,
    pub total_transmitted: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct NetworksStats {
    pub networks: Vec<NetworkStats>,
}
foxglove::static_typed_channel!(pub(crate) NETWORKS, "/networks", NetworksStats);

// Processes
#[derive(Debug, Serialize, JsonSchema)]
pub struct ProcessStats {
    pub pid: u32,
    pub parent_pid: String,
    pub name: String,
    pub status: String,
    pub cpu_usage: f32,
    pub memory_usage_kb: u64,
    pub start_time_seconds: u64,
    pub run_time_seconds: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct ProcessesStats {
    pub processes: Vec<ProcessStats>,
}
foxglove::static_typed_channel!(pub(crate) PROCESSES, "/processes", ProcessesStats);

// System
#[derive(Debug, Serialize, JsonSchema)]
pub struct SystemStats {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub os_long_version: String,
    pub host_name: String,
    pub kernel: String,
    pub boot_time_seconds: u64,
    pub uptime_seconds: u64,
    pub load_avg_one: f64,
    pub load_avg_five: f64,
    pub load_avg_fifteen: f64,
}
foxglove::static_typed_channel!(pub(crate) SYSTEM, "/system", SystemStats);
