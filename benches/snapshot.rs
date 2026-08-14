use std::hint::black_box;

use syspeek::{
    cli::ProcessSort,
    collect::{CollectionOptions, Collector},
    model::Scope,
};

fn main() {
    divan::main();
}

#[divan::bench]
fn collect_cpu_snapshot() -> syspeek::model::Snapshot {
    let mut collector = Collector::new(CollectionOptions {
        scope: Scope::Cpu,
        process_limit: 10,
        process_sort: ProcessSort::Cpu,
    });
    black_box(collector.collect())
}

#[divan::bench]
fn collect_process_snapshot() -> syspeek::model::Snapshot {
    let mut collector = Collector::new(CollectionOptions {
        scope: Scope::Processes,
        process_limit: 10,
        process_sort: ProcessSort::Cpu,
    });
    black_box(collector.collect())
}
