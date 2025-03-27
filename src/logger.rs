use sysinfo::{Components, Disks, Networks, ProcessesToUpdate, System};

use crate::Cli;

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

/// Collection of loggers that share a single System instance
pub struct LoggerCollection {
    system: System,
    cpu_enabled: bool,
    memory_enabled: bool,
    temperature: Option<Components>,
    disks: Option<Disks>,
    networks: Option<Networks>,
    processes_enabled: bool,
    system_enabled: bool,
}

impl LoggerCollection {
    pub fn new(args: &Cli) -> Self {
        let system = System::new_all();

        Self {
            system,
            cpu_enabled: args.cpu,
            memory_enabled: args.memory,
            temperature: if args.temperature {
                Some(Components::new_with_refreshed_list())
            } else {
                None
            },
            disks: if args.disks {
                Some(Disks::new_with_refreshed_list())
            } else {
                None
            },
            networks: if args.networks {
                Some(Networks::new_with_refreshed_list())
            } else {
                None
            },
            processes_enabled: args.processes,
            system_enabled: args.system,
        }
    }

    pub fn log_all(&mut self) {
        if self.cpu_enabled {
            self.system.refresh_cpu_all();
            CPU.log(&CpuStats {
                usage: self.system.global_cpu_usage(),
                physical_cores: System::physical_core_count(&self.system)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "Unknown".to_owned())
                    .trim()
                    .parse()
                    .unwrap_or(0),
                cores: self
                    .system
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

        if self.memory_enabled {
            self.system.refresh_memory();
            MEMORY.log(&MemoryStats {
                total_kb: self.system.total_memory(),
                available_kb: self.system.available_memory(),
                used_kb: self.system.used_memory(),
                swap_total_kb: self.system.total_swap(),
                swap_used_kb: self.system.used_swap(),
            });
        }

        if let Some(components) = &mut self.temperature {
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

        if let Some(disks) = &mut self.disks {
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

        if let Some(networks) = &mut self.networks {
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

        if self.processes_enabled {
            self.system.refresh_processes(ProcessesToUpdate::All, true);
            PROCESSES.log(&ProcessesStats {
                processes: self
                    .system
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

        if self.system_enabled {
            SYSTEM.log(&SystemStats {
                name: System::name().unwrap_or_else(|| "<unknown>".to_owned()),
                kernel_version: System::kernel_version().unwrap_or_else(|| "<unknown>".to_owned()),
                os_version: System::os_version().unwrap_or_else(|| "<unknown>".to_owned()),
                os_long_version: System::long_os_version()
                    .unwrap_or_else(|| "<unknown>".to_owned()),
                host_name: System::host_name().unwrap_or_else(|| "<unknown>".to_owned()),
                kernel: System::kernel_version().unwrap_or_else(|| "<unknown>".to_owned()),
                boot_time_seconds: System::boot_time(),
                uptime_seconds: System::uptime(),
                load_avg_one: System::load_average().one,
                load_avg_five: System::load_average().five,
                load_avg_fifteen: System::load_average().fifteen,
            });
        }
    }
}
