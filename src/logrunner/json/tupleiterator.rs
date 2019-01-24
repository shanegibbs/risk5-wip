use super::json::*;

// Takes batch of log lines until "mark" is reached
pub(crate) struct TupleIterator<I> {
    line_it: I,
}

impl<I> TupleIterator<I> {
    pub fn new(mut it: I) -> Self
    where
        I: Iterator<Item = (usize, LogLine, String)>,
    {
        // let mut it = LineIterator::new()?;
        loop {
            match it.next() {
                None => break,
                Some((_, LogLine::Mark, _)) => break,
                _ => (),
            }
        }

        TupleIterator { line_it: it }
    }
}

impl<I> Iterator for TupleIterator<I>
where
    I: Iterator<Item = (usize, LogLine, String)>,
{
    type Item = JsonLogTuple;

    fn next(&mut self) -> Option<JsonLogTuple> {
        let mut lines = vec![];

        let mut count = None;
        let mut insn = None;
        let mut state = None;
        let mut store = None;
        let mut mems = vec![];

        loop {
            let next = match self.line_it.next() {
                Some((c, ll, line)) => {
                    if count.is_none() {
                        count = Some(c)
                    }
                    lines.push(line);
                    ll
                }
                None => return None,
            };

            match next {
                LogLine::Mark => break,
                LogLine::Insn(n) => insn = Some(n),
                LogLine::State(n) => state = Some(n),
                LogLine::Memory(n) => mems.push(n),
                LogLine::Load(_) => (),
                LogLine::Store(n) => store = Some(n),
            }
        }

        // let insn = insn.expect(&format!("insn. line {}", self.line_it.count));
        let state = state.expect("state");

        for line in lines {
            trace!("Log {}", line);
        }

        Some(JsonLogTuple {
            line: count.unwrap_or(0),
            state,
            insn,
            store,
            mems,
        })
    }
}
