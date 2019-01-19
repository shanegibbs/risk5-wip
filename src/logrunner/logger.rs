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
    buffer: RwLock<Vec<(Level, String)>>,
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

fn print_line(level: &Level, line: &String) {
    writeln!(
        io::stderr(),
        "{0}[1m{0}[{1}m{2}{0}[0m {0}[{1}m{3}{0}[0m",
        27 as char,
        match level {
            Level::Error => "31",
            Level::Warn => "33",
            Level::Info => "34",
            Level::Debug => "35",
            Level::Trace => "36",
        },
        level,
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
            for (level, line) in buffer.iter() {
                print_line(level, line);
            }
            buffer.clear();
        } else {
            buffer.push((record.metadata().level(), to_line(record)));
            if buffer.len() > 200 {
                buffer.remove(0);
            }
        }

        if record.metadata().level() <= Level::Warn {
            print_line(&record.metadata().level(), &to_line(record));
        }
    }

    fn flush(&self) {}
}