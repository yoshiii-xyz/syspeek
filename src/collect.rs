use std::{
    cmp::Ordering,
    ffi::OsStr,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use sysinfo::{
    CpuRefreshKind, Disks, MemoryRefreshKind, Networks, ProcessRefreshKind, RefreshKind, System,
    UpdateKind,
};

use crate::{
    cli::ProcessSort,
    model::{
        CpuInfo, LoadAverage, MemoryInfo, NetworkAddress, NetworkInfo, NetworkInterface,
        ProcessInfo, ProcessRecord, SCHEMA_VERSION, Scope, Snapshot, StorageInfo, StorageVolume,
        SwapInfo, SystemInfo, Warning,
    },
};

const CPU_SAMPLE_WAIT: Duration = Duration::from_millis(200);

#[derive(Clone, Copy, Debug)]
pub struct CollectionOptions {
    pub scope: Scope,
    pub process_limit: usize,
    pub process_sort: ProcessSort,
}

pub struct Collector {
    system: System,
    disks: Option<Disks>,
    networks: Option<Networks>,
    options: CollectionOptions,
    refresh_kind: RefreshKind,
    sample_ready: bool,
}

impl Collector {
    pub fn new(options: CollectionOptions) -> Self {
        let refresh_kind = refresh_kind(options.scope);
        let system = System::new_with_specifics(refresh_kind);
        let disks = options.scope.includes_disk().then(Disks::new_with_refreshed_list);
        let networks = options.scope.includes_network().then(Networks::new_with_refreshed_list);

        Self {
            system,
            disks,
            networks,
            options,
            refresh_kind,
            sample_ready: !options.scope.includes_cpu() && !options.scope.includes_processes(),
        }
    }

    pub fn collect(&mut self) -> Snapshot {
        if !self.sample_ready {
            thread::sleep(CPU_SAMPLE_WAIT);
            self.refresh();
            self.sample_ready = true;
        } else {
            self.refresh();
        }

        let mut warnings = Vec::new();
        if !sysinfo::IS_SUPPORTED_SYSTEM {
            warnings.push(Warning {
                section: "system".to_string(),
                message: "sysinfo does not provide native support for this target".to_string(),
            });
        }

        let cpu = self.options.scope.includes_cpu().then(|| self.collect_cpu());
        let cpu = cpu.flatten();
        if self.options.scope.includes_cpu() && cpu.is_none() {
            warnings.push(unavailable_warning("cpu", "CPU details were not reported"));
        }

        let memory = self.options.scope.includes_memory().then(|| self.collect_memory()).flatten();
        if self.options.scope.includes_memory() && memory.is_none() {
            warnings.push(unavailable_warning("memory", "memory details were not reported"));
        }

        let storage = self.options.scope.includes_disk().then(|| self.collect_storage()).flatten();
        if let Some(storage) = &storage
            && storage.volumes.is_empty()
        {
            warnings.push(unavailable_warning("disk", "no mounted volumes were reported"));
        }

        let network =
            self.options.scope.includes_network().then(|| self.collect_network()).flatten();
        if let Some(network) = &network
            && network.interfaces.is_empty()
        {
            warnings.push(unavailable_warning("network", "no network interfaces were reported"));
        }

        let processes = self.options.scope.includes_processes().then(|| self.collect_processes());
        if let Some(processes) = &processes
            && processes.total_count == 0
        {
            warnings.push(unavailable_warning("processes", "no processes were reported"));
        }

        Snapshot {
            schema_version: SCHEMA_VERSION,
            collected_at_unix_seconds: unix_timestamp(),
            scope: self.options.scope,
            system: self.collect_system(),
            cpu,
            memory,
            storage,
            network,
            processes,
            warnings,
        }
    }

    fn refresh(&mut self) {
        self.system.refresh_specifics(self.refresh_kind);
        if let Some(disks) = &mut self.disks {
            disks.refresh(true);
        }
        if let Some(networks) = &mut self.networks {
            networks.refresh(true);
        }
    }

    fn collect_system(&self) -> SystemInfo {
        SystemInfo {
            platform: clean(System::name()),
            distribution: clean(Some(System::distribution_id())),
            os_version: clean(System::long_os_version().or_else(System::os_version)),
            kernel_version: clean(System::kernel_version()),
            architecture: clean(Some(System::cpu_arch())),
            hostname: clean(System::host_name()),
            uptime_seconds: Some(System::uptime()),
            boot_time_unix_seconds: non_zero(System::boot_time()),
            load_average: load_average(),
        }
    }

    fn collect_cpu(&self) -> Option<CpuInfo> {
        let cpus = self.system.cpus();
        if cpus.is_empty() {
            return None;
        }

        let model =
            cpus.iter().find_map(|cpu| clean_str(cpu.brand()).or_else(|| clean_str(cpu.name())));
        let average_frequency_mhz = {
            let frequencies: Vec<u64> =
                cpus.iter().map(|cpu| cpu.frequency()).filter(|frequency| *frequency > 0).collect();
            if frequencies.is_empty() {
                None
            } else {
                Some(frequencies.iter().sum::<u64>() / frequencies.len() as u64)
            }
        };

        Some(CpuInfo {
            model,
            architecture: clean(Some(System::cpu_arch())),
            physical_cores: System::physical_core_count().filter(|count| *count > 0),
            logical_processors: Some(cpus.len()),
            utilization_percent: percent(self.system.global_cpu_usage()),
            average_frequency_mhz,
            load_average: load_average(),
        })
    }

    fn collect_memory(&self) -> Option<MemoryInfo> {
        let total = self.system.total_memory();
        if total == 0 {
            return None;
        }
        let used = self.system.used_memory();
        let available = self.system.available_memory();
        let total_swap = self.system.total_swap();
        let used_swap = self.system.used_swap();
        let free_swap = self.system.free_swap();

        Some(MemoryInfo {
            total_bytes: Some(total),
            used_bytes: Some(used),
            available_bytes: Some(available),
            utilization_percent: utilization(used, total),
            swap: Some(SwapInfo {
                total_bytes: Some(total_swap),
                used_bytes: Some(used_swap),
                free_bytes: Some(free_swap),
                utilization_percent: utilization(used_swap, total_swap),
            }),
        })
    }

    fn collect_storage(&self) -> Option<StorageInfo> {
        let disks = self.disks.as_ref()?;
        let mut volumes = disks
            .list()
            .iter()
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);
                StorageVolume {
                    mount_point: display_path(disk.mount_point()),
                    device: clean_os(disk.name()),
                    filesystem: clean_os(disk.file_system()),
                    kind: Some(disk.kind().to_string()),
                    total_bytes: Some(total),
                    used_bytes: Some(used),
                    available_bytes: Some(available),
                    utilization_percent: utilization(used, total),
                    removable: Some(disk.is_removable()),
                    read_only: Some(disk.is_read_only()),
                }
            })
            .collect::<Vec<_>>();
        volumes.sort_by(|left, right| left.mount_point.cmp(&right.mount_point));
        Some(StorageInfo { volumes })
    }

    fn collect_network(&self) -> Option<NetworkInfo> {
        let networks = self.networks.as_ref()?;
        let mut interfaces = networks
            .list()
            .iter()
            .map(|(name, network)| NetworkInterface {
                name: name.clone(),
                state: Some(network.operational_state().to_string()),
                mtu: non_zero(network.mtu()),
                mac_address: (!network.mac_address().is_unspecified())
                    .then(|| network.mac_address().to_string()),
                addresses: network
                    .ip_networks()
                    .iter()
                    .map(|address| NetworkAddress {
                        address: address.addr.to_string(),
                        prefix_length: address.prefix,
                    })
                    .collect(),
                total_received_bytes: Some(network.total_received()),
                total_transmitted_bytes: Some(network.total_transmitted()),
            })
            .collect::<Vec<_>>();
        interfaces.sort_by(|left, right| left.name.cmp(&right.name));
        Some(NetworkInfo { interfaces })
    }

    fn collect_processes(&self) -> ProcessInfo {
        let mut processes = self
            .system
            .processes()
            .values()
            .map(|process| ProcessRecord {
                pid: process.pid().as_u32(),
                name: process_name(process.name()),
                executable: process.exe().map(display_path),
                status: Some(format!("{:?}", process.status())),
                cpu_percent: percent(process.cpu_usage()),
                resident_memory_bytes: Some(process.memory()),
                virtual_memory_bytes: Some(process.virtual_memory()),
            })
            .collect::<Vec<_>>();

        processes.sort_by(|left, right| {
            let primary = match self.options.process_sort {
                ProcessSort::Cpu => right
                    .cpu_percent
                    .unwrap_or_default()
                    .total_cmp(&left.cpu_percent.unwrap_or_default()),
                ProcessSort::Memory => right
                    .resident_memory_bytes
                    .unwrap_or_default()
                    .cmp(&left.resident_memory_bytes.unwrap_or_default()),
                ProcessSort::Pid => left.pid.cmp(&right.pid),
                ProcessSort::Name => left.name.cmp(&right.name),
            };
            if primary == Ordering::Equal { left.pid.cmp(&right.pid) } else { primary }
        });

        let total_count = processes.len();
        processes.truncate(self.options.process_limit);
        ProcessInfo { total_count, processes }
    }
}

fn refresh_kind(scope: Scope) -> RefreshKind {
    let mut refresh = RefreshKind::nothing();
    if scope.includes_cpu() {
        refresh = refresh.with_cpu(CpuRefreshKind::everything());
    }
    if scope.includes_memory() {
        refresh = refresh.with_memory(MemoryRefreshKind::everything());
    }
    if scope.includes_processes() {
        let process_refresh = ProcessRefreshKind::nothing()
            .with_cpu()
            .with_memory()
            .with_exe(UpdateKind::OnlyIfNotSet)
            .without_tasks();
        refresh = refresh.with_processes(process_refresh);
    }
    refresh
}

fn load_average() -> Option<LoadAverage> {
    if cfg!(windows) {
        return None;
    }
    let load = System::load_average();
    Some(LoadAverage {
        one_minute: load.one,
        five_minutes: load.five,
        fifteen_minutes: load.fifteen,
    })
}

fn clean(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn clean_str(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn clean_os(value: &OsStr) -> Option<String> {
    clean(Some(value.to_string_lossy().into_owned()))
}

fn display_path(value: &std::path::Path) -> String {
    value.to_string_lossy().into_owned()
}

fn process_name(value: &OsStr) -> String {
    let name = value.to_string_lossy().trim().to_string();
    if name.is_empty() { "unknown".to_string() } else { name }
}

fn percent(value: f32) -> Option<f32> {
    value.is_finite().then(|| (value.max(0.0) * 10.0).round() / 10.0)
}

fn utilization(used: u64, total: u64) -> Option<f32> {
    if total == 0 {
        None
    } else {
        percent((used as f64 / total as f64 * 100.0) as f32).map(|value| value.min(100.0))
    }
}

fn non_zero(value: u64) -> Option<u64> {
    (value > 0).then_some(value)
}

fn unix_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn unavailable_warning(section: &str, message: &str) -> Warning {
    Warning { section: section.to_string(), message: message.to_string() }
}
