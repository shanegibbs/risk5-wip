use super::*;
use std::io::BufRead;

pub(crate) struct LogLineIterator {
    pub(crate) count: usize,
    pub(crate) had_first_state: bool,
    pub(crate) lines: Lines<BufReader<io::Stdin>>,
}

impl LogLineIterator {
    pub fn new() -> Result<Self, io::Error> {
        let reader = BufReader::new(io::stdin());
        Ok(LogLineIterator {
            count: 0,
            had_first_state: false,
            lines: reader.lines(),
        })
    }
}

impl Iterator for LogLineIterator {
    type Item = (LogLine, String);

    fn next(&mut self) -> Option<(LogLine, String)> {
        loop {
            let line = match self.lines.next() {
                Some(l) => l,
                None => return None,
            };

            self.count += 1;

            let l = line.expect("line");

            if l.is_empty() {
                continue;
            }

            let d: LogLine = match serde_json::from_str(l.as_str()) {
                Ok(l) => l,
                Err(e) => {
                    error!("Parsing line: {}", e);
                    error!("{}", l);
                    panic!("line parse failed");
                }
            };

            if let LogLine::State(_) = d {
                self.had_first_state = true;
            } else {
                if !self.had_first_state {
                    continue;
                }
            }

            return Some((d, l));
        }
    }
}
