use std::io::{self, IsTerminal, Write};

use clap::Parser;

use syspeek::{
    cli::{Cli, Command, ProcessSort},
    collect::{CollectionOptions, Collector},
    model::Scope,
    render::{HumanRenderOptions, render_human, render_json},
};

fn main() {
    let cli = Cli::parse();
    let stdout_is_terminal = io::stdout().is_terminal();
    if cli.watch && cli.json {
        usage_error("--watch cannot be combined with --json");
    }
    if cli.watch && !stdout_is_terminal {
        usage_error(
            "--watch requires an interactive terminal; remove --watch for redirected output",
        );
    }

    let scope = cli.command.as_ref().map_or(Scope::All, Command::scope);
    let (process_limit, process_sort) =
        cli.command.as_ref().map_or((10, ProcessSort::Cpu), Command::process_options);
    let mut collector = Collector::new(CollectionOptions { scope, process_limit, process_sort });
    let render_options =
        HumanRenderOptions { color: cli.color, ascii: cli.ascii, stdout_is_terminal, process_sort };

    if cli.watch {
        if let Err(error) = syspeek::watch::run(collector, render_options, cli.interval) {
            eprintln!("syspeek: {error}");
            std::process::exit(1);
        }
        return;
    }

    let snapshot = collector.collect();
    let result = if cli.json {
        render_json(&snapshot).map_err(|error| error.to_string())
    } else {
        Ok(render_human(&snapshot, render_options))
    };
    match result {
        Ok(output) => {
            let mut stdout = io::stdout();
            match stdout.write_all(output.as_bytes()).and_then(|_| stdout.flush()) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::BrokenPipe => (),
                Err(error) => {
                    eprintln!("syspeek: could not write output: {error}");
                    std::process::exit(1);
                }
            }
        }
        Err(error) => {
            eprintln!("syspeek: could not serialize JSON: {error}");
            std::process::exit(1);
        }
    }
}

fn usage_error(message: &str) -> ! {
    eprintln!("syspeek: {message}");
    std::process::exit(2);
}
