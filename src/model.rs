use serde::Serialize;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    All,
    System,
    Cpu,
    Memory,
    Disk,
    Network,
    Processes,
}

impl Scope {
    pub const fn includes_cpu(self) -> bool {
        matches!(self, Self::All | Self::Cpu)
    }

    pub const fn includes_memory(self) -> bool {
        matches!(self, Self::All | Self::Memory)
    }

    pub const fn includes_disk(self) -> bool {
        matches!(self, Self::All | Self::Disk)
    }

    pub const fn includes_network(self) -> bool {
        matches!(self, Self::All | Self::Network)
    }

    pub const fn includes_processes(self) -> bool {
        matches!(self, Self::All | Self::Processes)
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::All => "all",
            Self::System => "system",
            Self::Cpu => "cpu",
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::Network => "network",
            Self::Processes => "processes",
        };
        formatter.write_str(value)
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub schema_version: u32,
    pub collected_at_unix_seconds: u64,
    pub scope: Scope,
    pub system: SystemInfo,
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub storage: Option<StorageInfo>,
    pub network: Option<NetworkInfo>,
    pub processes: Option<ProcessInfo>,
    pub warnings: Vec<Warning>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub platform: Option<String>,
    pub distribution: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub architecture: Option<String>,
    pub hostname: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub boot_time_unix_seconds: Option<u64>,
    pub load_average: Option<LoadAverage>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub model: Option<String>,
    pub architecture: Option<String>,
    pub physical_cores: Option<usize>,
    pub logical_processors: Option<usize>,
    pub utilization_percent: Option<f32>,
    pub average_frequency_mhz: Option<u64>,
    pub load_average: Option<LoadAverage>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadAverage {
    pub one_minute: f64,
    pub five_minutes: f64,
    pub fifteen_minutes: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
    pub utilization_percent: Option<f32>,
    pub swap: Option<SwapInfo>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    pub total_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub free_bytes: Option<u64>,
    pub utilization_percent: Option<f32>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub volumes: Vec<StorageVolume>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageVolume {
    pub mount_point: String,
    pub device: Option<String>,
    pub filesystem: Option<String>,
    pub kind: Option<String>,
    pub total_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
    pub utilization_percent: Option<f32>,
    pub removable: Option<bool>,
    pub read_only: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfo {
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub name: String,
    pub state: Option<String>,
    pub mtu: Option<u64>,
    pub mac_address: Option<String>,
    pub addresses: Vec<NetworkAddress>,
    pub total_received_bytes: Option<u64>,
    pub total_transmitted_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkAddress {
    pub address: String,
    pub prefix_length: u8,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub total_count: usize,
    pub processes: Vec<ProcessRecord>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessRecord {
    pub pid: u32,
    pub name: String,
    pub executable: Option<String>,
    pub status: Option<String>,
    pub cpu_percent: Option<f32>,
    pub resident_memory_bytes: Option<u64>,
    pub virtual_memory_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Warning {
    pub section: String,
    pub message: String,
}
