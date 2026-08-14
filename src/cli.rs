use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};

use crate::model::Scope;

#[derive(Debug, Parser)]
#[command(
    name = "syspeek",
    version,
    about = "Inspect the current machine with a fast, readable diagnostics snapshot",
    after_help = "Run `syspeek <command> --help` for command-specific options."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Emit the stable machine-readable JSON schema instead of terminal output.
    #[arg(short = 'j', long, global = true)]
    pub json: bool,

    /// Select automatic, always-on, or disabled ANSI color output.
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto, global = true)]
    pub color: ColorChoice,

    /// Use ASCII section separators for terminals with limited Unicode support.
    #[arg(long, global = true)]
    pub ascii: bool,

    /// Refresh the human-readable report continuously until interrupted.
    #[arg(long, global = true)]
    pub watch: bool,

    /// Delay between watch-mode refreshes, such as 500ms, 2s, or 1m.
    #[arg(
        long,
        value_name = "DURATION",
        value_parser = parse_duration,
        default_value = "2s",
        global = true,
        requires = "watch"
    )]
    pub interval: Duration,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Show operating system and host details.
    System,
    /// Show CPU model, topology, usage, and frequency.
    Cpu,
    /// Show RAM and swap statistics.
    Memory,
    /// Show mounted filesystem capacity.
    Disk,
    /// Show local interfaces, addresses, and counters.
    Network,
    /// Show the busiest local processes without modifying them.
    Processes {
        /// Maximum number of processes to display.
        #[arg(short = 'n', long, value_name = "COUNT", value_parser = parse_positive_usize, default_value = "10")]
        limit: usize,
        /// Sort by CPU, memory, PID, or name.
        #[arg(long, value_enum, default_value_t = ProcessSort::Cpu)]
        sort: ProcessSort,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ProcessSort {
    Cpu,
    Memory,
    Pid,
    Name,
}

impl Command {
    pub const fn scope(&self) -> Scope {
        match self {
            Self::System => Scope::System,
            Self::Cpu => Scope::Cpu,
            Self::Memory => Scope::Memory,
            Self::Disk => Scope::Disk,
            Self::Network => Scope::Network,
            Self::Processes { .. } => Scope::Processes,
        }
    }

    pub const fn process_options(&self) -> (usize, ProcessSort) {
        match self {
            Self::Processes { limit, sort } => (*limit, *sort),
            _ => (10, ProcessSort::Cpu),
        }
    }
}

fn parse_positive_usize(input: &str) -> Result<usize, String> {
    let value =
        input.parse::<usize>().map_err(|_| format!("{input:?} is not a positive integer"))?;
    if value == 0 {
        return Err("count must be greater than zero".to_string());
    }
    Ok(value)
}

fn parse_duration(input: &str) -> Result<Duration, String> {
    let normalized = input.trim().to_ascii_lowercase();
    let split = normalized
        .find(|character: char| !character.is_ascii_digit() && character != '.')
        .unwrap_or(normalized.len());
    let (number, unit) = normalized.split_at(split);
    let value = number.parse::<f64>().map_err(|_| format!("invalid duration {input:?}"))?;
    if !value.is_finite() || value <= 0.0 {
        return Err("duration must be greater than zero".to_string());
    }
    let seconds = match unit {
        "ms" => value / 1_000.0,
        "s" | "" => value,
        "m" => value * 60.0,
        _ => return Err(format!("unknown duration unit in {input:?}; use ms, s, or m")),
    };
    let duration = Duration::from_secs_f64(seconds);
    if duration < Duration::from_millis(200) {
        return Err("watch interval must be at least 200ms".to_string());
    }
    Ok(duration)
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
