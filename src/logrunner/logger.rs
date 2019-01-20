use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};
use std::io::{self, Write};
use std::sync::*;

lazy_static! {
    static ref LOGGER: TracerLogger = { TracerLogger::new() };
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Trace))
}

struct TracerLogger {
    buffer: RwLock<Vec<(Level, Option<String>, String)>>,
}

impl TracerLogger {
    fn new() -> Self {
        TracerLogger {
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
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
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
