use super::*;

pub(crate) struct LogTupleDedupIterator {
    it: LogTupleIterator,
    buf: Option<LogTuple>,
}

impl LogTupleDedupIterator {
    fn new() -> Result<Self, io::Error> {
        Ok(LogTupleDedupIterator {
            it: LogTupleIterator::new()?,
            buf: None,
        })
    }
}

impl Iterator for LogTupleDedupIterator {
    type Item = LogTuple;

    fn next(&mut self) -> Option<LogTuple> {
        if self.buf.is_none() {
            let n = self.it.next();
            if n.is_none() {
                return None;
            }
            self.buf = n;
            return self.next();
        }

        let buf = self.buf.take().expect("buf");
        let n = match self.it.next() {
            Some(n) => n,
            None => return Some(buf),
        };

        if n.state.pc == buf.state.pc {
            panic!("duplicate insn pc");
            // merge
            n.mems.extend(buf.mems);
            return Some(n);
        } else {
            self.buf = Some(n);
            return Some(buf);
        }
    }
}
