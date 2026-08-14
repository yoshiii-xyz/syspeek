use std::{
    io::{self, IsTerminal, Write},
    thread,
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};

use crate::{
    collect::Collector,
    render::{HumanRenderOptions, render_human},
};

pub fn run(
    mut collector: Collector,
    options: HumanRenderOptions,
    interval: Duration,
) -> io::Result<()> {
    if !io::stdout().is_terminal() {
        return Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "watch mode requires an interactive terminal; use a normal snapshot for redirected output",
        ));
    }

    let mut stdout = io::stdout();
    loop {
        let snapshot = collector.collect();
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        stdout.write_all(render_human(&snapshot, options).as_bytes())?;
        stdout.flush()?;
        thread::sleep(interval);
    }
}
