use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};
use std::env;
use std::io::{self, Write};
use std::sync::*;

lazy_static! {
    static ref LOGGER: TracerLogger = { TracerLogger::new() };
}

pub fn level_override() -> Option<LevelFilter> {
    env::var("LOG").ok().map(|s| match s.as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Warn,
    })
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(level_override().unwrap_or(LevelFilter::Trace)))
}

// Keeps a window of all levels of logs. Prints
// and flushes the buffer on receiving an error.
struct TracerLogger {
    level: Option<LevelFilter>,
    buffer: RwLock<Vec<(Level, Option<String>, String)>>,
}

impl TracerLogger {
    fn new() -> Self {
        TracerLogger {
            level: level_override(),
            buffer: RwLock::new(vec![]),
        }
    }
}

fn to_line(record: &Record) -> String {
    format!("{}", record.args())
}

fn print_line(level: &Level, module: Option<&str>, line: &String) {
    let module = module.unwrap_or("");
    let module = format!("{0}[2m{1: <25}{0}[22m", 27 as char, module);
    writeln!(
        io::stderr(),
        "{0}[1m{0}[{1}m{2:>5}{0}[0m {3} {0}[{1}m{4}{0}[0m",
        27 as char,
        match level {
            Level::Error => "38;5;124",
            Level::Warn => "38;5;208",
            Level::Info => "38;5;40",
            Level::Debug => "35",
            Level::Trace => "36",
        },
        level,
        module,
        line
    )
    .expect("print log line");
}

impl log::Log for LOGGER {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.level.map(|l| l >= metadata.level()).unwrap_or(true)
    }

    fn log(&self, record: &Record) {
        if self.level.is_some() {
            print_line(
                &record.metadata().level(),
                record.module_path(),
                &to_line(record),
            );
            return;
        }

        let mut buffer = self.buffer.write().expect("log lock");

        if record.metadata().level() == Level::Error {
            for (level, module, line) in buffer.iter() {
                print_line(level, module.as_ref().map(|s| s.as_str()), line);
            }
            buffer.clear();
        } else {
            buffer.push((
                record.metadata().level(),
                record.module_path().map(|s| s.into()),
                to_line(record),
            ));
            if buffer.len() > 200 {
                buffer.remove(0);
            }
        }

        if record.metadata().level() <= Level::Warn {
            print_line(
                &record.metadata().level(),
                record.module_path(),
                &to_line(record),
            );
        }
    }

    fn flush(&self) {}
}
