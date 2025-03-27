/// Collection of loggers that share a single System instance
use sysinfo::{Components, Disks, Networks, ProcessesToUpdate, System};

use crate::{Cli, channels::*};

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
            self.log_cpu();
        }
        if self.memory_enabled {
            self.log_memory();
        }
        if let Some(components) = self.temperature.take() {
            self.log_temperature(components);
        }
        if let Some(disks) = self.disks.take() {
            self.log_disks(disks);
        }
        if let Some(networks) = self.networks.take() {
            self.log_networks(networks);
        }
        if self.processes_enabled {
            self.log_processes();
        }
        if self.system_enabled {
            self.log_system();
        }
    }

    pub fn log_cpu(&mut self) {
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

    pub fn log_memory(&mut self) {
        self.system.refresh_memory();
        MEMORY.log(&MemoryStats {
            total_kb: self.system.total_memory(),
            available_kb: self.system.available_memory(),
            used_kb: self.system.used_memory(),
            swap_total_kb: self.system.total_swap(),
            swap_used_kb: self.system.used_swap(),
        });
    }

    pub fn log_temperature(&mut self, mut components: Components) {
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
        self.temperature = Some(components);
    }

    pub fn log_disks(&mut self, mut disks: Disks) {
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
        self.disks = Some(disks);
    }

    pub fn log_networks(&mut self, mut networks: Networks) {
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
        self.networks = Some(networks);
    }

    pub fn log_processes(&mut self) {
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

    pub fn log_system(&self) {
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
}
