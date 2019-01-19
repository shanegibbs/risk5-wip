use super::loglineiterator::LogLineIterator;
use super::*;

pub(crate) struct JsonLogTupleIterator {
    line_it: LogLineIterator,
}

impl JsonLogTupleIterator {
    pub fn new() -> Result<Self, io::Error> {
        let mut it = LogLineIterator::new()?;
        loop {
            match it.next() {
                None => break,
                Some((LogLine::Mark, _)) => break,
                _ => (),
            }
        }

        Ok(JsonLogTupleIterator { line_it: it })
    }
}

impl Iterator for JsonLogTupleIterator {
    type Item = JsonLogTuple;

    fn next(&mut self) -> Option<JsonLogTuple> {
        let mut lines = vec![];

        let mut insn = None;
        let mut state = None;
        let mut mems = vec![];

        loop {
            let next = match self.line_it.next() {
                Some((ll, line)) => {
                    lines.push(line);
                    ll
                }
                None => return None,
            };

            match next {
                LogLine::Mark => {
                    // if insn.is_some() {
                    break;
                    // }
                }
                LogLine::Insn(n) => insn = Some(n),
                LogLine::State(n) => state = Some(n),
                LogLine::Memory(n) => mems.push(n),
                LogLine::Load(_) => (),
                LogLine::Store(_) => (),
            }
        }

        // let insn = insn.expect(&format!("insn. line {}", self.line_it.count));
        let state = state.expect("state");

        for line in lines {
            trace!("Log {}", line);
        }

        Some(JsonLogTuple {
            line: self.line_it.count,
            state: state,
            insn,
            mems,
        })
    }
}
