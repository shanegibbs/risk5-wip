use super::*;
use std::io::BufRead;

pub(crate) struct LineIterator {
    pub(crate) count: usize,
    pub(crate) had_first_state: bool,
    pub(crate) lines: Lines<BufReader<io::Stdin>>,
}

impl LineIterator {
    pub fn new() -> Self {
        let reader = BufReader::new(io::stdin());
        LineIterator {
            count: 0,
            had_first_state: false,
            lines: reader.lines(),
        }
    }
}

impl Iterator for LineIterator {
    type Item = (usize, LogLine, String);

    fn next(&mut self) -> Option<(usize, LogLine, String)> {
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
            } else if !self.had_first_state {
                continue;
            }

            return Some((self.count, d, l));
        }
    }
}
