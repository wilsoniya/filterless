use std::collections::VecDeque;
use std::iter::{Iterator, repeat};

use super::line_buffer::LineBuffer;
use super::iter::{ContextLine, FilteredLine, FilterPredicate, Gap};

/// Buffer for providing visibility into past, present, and future lines
/// produced by an iterator.
///
/// Internally it stores a deque containing lines read from an underlying
/// iterator, with the deque taking the size 2 * `context_lines` + 1. This size
/// allows the deque to store `context_lines` past lines, one line
/// representing the "current" line, and `context_lines` future lines. New
/// lines are pushed to the back of the deque and old lines are popped from the
/// beginning. In this way the "current" line always resides in the exact
/// middle of the deque.
pub struct ContextBuffer<T: Iterator<Item=String>> {
    filter_predicate: Option<FilterPredicate>,
    /// earlier lines in lower indexes
    buffer: VecDeque<Option<ContextLine>>,
    /// underlying iterator
    iter: LineBuffer<T>,
    gap: Gap
}

impl<T: Iterator<Item=String>> ContextBuffer<T> {
    pub fn new(filter_predicate: Option<FilterPredicate>,
           mut iter: LineBuffer<T>) -> ContextBuffer<T> {

        let buffer = match filter_predicate {
            Some(FilterPredicate{ ref filter_string, ref context_lines }) => {
                let capacity = context_lines * 2 + 1;
                repeat(None)
                    .take(context_lines + 1)
                    .chain((&mut iter).map(|numbered_line| {
                        Some(ContextLine::from_numbered_line(
                                numbered_line.to_owned(), &filter_string))
                    }))
                    .chain(repeat(None))
                    .take(capacity)
                    .collect()
            },
            None => {
                VecDeque::with_capacity(1)
            },
        };

        ContextBuffer {
            filter_predicate: filter_predicate,
            buffer: buffer,
            iter: iter,
            gap: Gap::None,
        }
    }

    /// Returns `True` if any lines in `buffer` match the filter.
    fn buffer_has_matches(&self) -> bool {
        self.buffer.iter()
            .map(|maybe_elt| {
                match maybe_elt {
                    &Some(ContextLine::Match(_)) => true,
                    _ => false,
                }
            })
            .any(|m| m)
    }

    fn fill_buffer(&mut self) {
        match self.filter_predicate {
            Some(FilterPredicate{ ref filter_string, .. }) => {
                let item = self.iter.next().map(|numbered_line| {
                    ContextLine::from_numbered_line(numbered_line.to_owned(),
                    &filter_string)
                });
                self.buffer.pop_front();
                self.buffer.push_back(item);

                while !self.buffer_has_matches() {
                    if let Some(numbered_line) = self.iter.next() {
                        let context_line = ContextLine::from_numbered_line(
                            numbered_line.to_owned(), &filter_string);

                        if let ContextLine::Match(_) = context_line {
                            self.gap = Gap::Current;
                        };

                        self.buffer.pop_front();
                        self.buffer.push_back(Some(context_line));
                    } else {
                        self.buffer.clear();
                        break;
                    }
                }
            },
            None => {
                self.buffer.pop_front();
                if let Some(numbered_line) = self.iter.next() {
                    let context_line = ContextLine::NoMatch(
                        numbered_line.to_owned());
                    self.buffer.push_back(Some(context_line));
                }
            }
        }
    }

    fn classify_cur_line(&self) -> Option<FilteredLine> {
        match self.filter_predicate {
            Some(FilterPredicate{ ref context_lines, .. }) => {
                let cur_idx = context_lines;
                self.buffer.get(*cur_idx)
                    .and_then(|maybe_context_line| {
                        maybe_context_line.as_ref().map(|context_line| {
                            context_line.to_filtered_line(&self.filter_predicate)
                        })
                    })
            },
            None => {
                let cur_idx = 0;
                self.buffer.get(cur_idx)
                    .and_then(|maybe_context_line| {
                        maybe_context_line.as_ref().map(|context_line| {
                            context_line.to_filtered_line(&self.filter_predicate)
                        })
                    })
            },
        }
    }

    /// Consumes this `ContextBuffer`, returning the inner `LineBuffer`.
    pub fn into_line_buffer(self) -> LineBuffer<T> {
        self.iter
    }
}

impl<T: Iterator<Item = String>> Iterator for ContextBuffer<T> {
    type Item = FilteredLine;

    fn next(&mut self) -> Option<Self::Item> {
        match self.gap {
            Gap::None => {
                self.fill_buffer();
                match self.gap {
                    Gap::Current => {
                        self.gap = Gap::Previous;
                        Some(FilteredLine::Gap)
                    },
                    _ => {
                        self.gap = Gap::None;
                        self.classify_cur_line()
                    },
                }
            },
            _ => {
                self.gap = Gap::None;
                self.classify_cur_line()
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::ContextBuffer;
    use iter::iter::FilteredLine;
    use iter::iter::FilterPredicate;
    use iter::line_buffer::LineBuffer;

    #[test]
    fn test1() {
        let lines: Vec<String> = vec![
            "none".to_owned(),
            "ctx".to_owned(),
            "ctx".to_owned(),
            "match".to_owned(),
            "ctx".to_owned(),
            "ctx".to_owned(),
            "none".to_owned(),
            "none".to_owned(),
            "ctx".to_owned(),
            "ctx".to_owned(),
            "match".to_owned(),
            "ctx".to_owned(),
        ];
        let context_lines = 2;
        let filter_string = "match".to_owned();
        let iter = lines.iter().map(|i| i.to_owned());
        let line_buf = LineBuffer::new(iter);

        let pred = FilterPredicate {
            filter_string: filter_string,
            context_lines: context_lines
        };
        let mut cb = ContextBuffer::new(Some(pred), line_buf);

        let e0 = cb.next();
        assert!(e0 == Some(FilteredLine::Gap));
        let e1 = cb.next();
        assert!(e1 == Some(FilteredLine::ContextLine((2, String::from("ctx")))));
        let e2 = cb.next();
        assert!(e2 == Some(FilteredLine::ContextLine((3, String::from("ctx")))));
        let e3 = cb.next();
        assert!(e3 == Some(FilteredLine::MatchLine((4, String::from("match")))));
        let e4 = cb.next();
        assert!(e4 == Some(FilteredLine::ContextLine((5, String::from("ctx")))));
        let e5 = cb.next();
        assert!(e5 == Some(FilteredLine::ContextLine((6, String::from("ctx")))));
        let e6 = cb.next();
        assert!(e6 == Some(FilteredLine::Gap));
        let e7 = cb.next();
        assert!(e7 == Some(FilteredLine::ContextLine((9, String::from("ctx")))));
        let e8 = cb.next();
        assert!(e8 == Some(FilteredLine::ContextLine((10, String::from("ctx")))));
        let e9 = cb.next();
        assert!(e9 == Some(FilteredLine::MatchLine((11, String::from("match")))));
        let e10 = cb.next();
        assert!(e10 == Some(FilteredLine::ContextLine((12, String::from("ctx")))));
    }

    #[test]
    fn test2() {
        let lines: Vec<String> = vec![
            "match".to_owned(),
            "match".to_owned(),
            "none".to_owned(),
            "match".to_owned(),
            "none".to_owned(),
        ];
        let context_lines = 0;
        let filter_string = "match".to_owned();
        let iter = lines.iter().map(|i| i.to_owned());
        let line_buf = LineBuffer::new(iter);

        let pred = FilterPredicate {
            filter_string: filter_string,
            context_lines: context_lines
        };
        let mut cb = ContextBuffer::new(Some(pred), line_buf);

        let e0 = cb.next();
        println!("{:?}", e0);
        assert!(e0 == Some(FilteredLine::MatchLine((1, String::from("match")))));
        let e1 = cb.next();
        assert!(e1 == Some(FilteredLine::MatchLine((2, String::from("match")))));
        let e2 = cb.next();
        assert!(e2 == Some(FilteredLine::Gap));
        let e3 = cb.next();
        assert!(e3 == Some(FilteredLine::MatchLine((4, String::from("match")))));
        let e4 = cb.next();
        assert!(e4 == None);
    }

    #[test]
    fn test3() {
        let lines: Vec<String> = vec![
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
        ];
        let iter = lines.iter().map(|i| i.to_owned());
        let line_buf = LineBuffer::new(iter);

        let mut cb = ContextBuffer::new(None, line_buf);

        let e1 = cb.next();
        println!("{:?}", e1);
        assert!(e1 == Some(FilteredLine::UnfilteredLine((1, String::from("one")))));
        let e2 = cb.next();
        assert!(e2 == Some(FilteredLine::UnfilteredLine((2, String::from("two")))));
        let e3 = cb.next();
        assert!(e3 == Some(FilteredLine::UnfilteredLine((3, String::from("three")))));
        let e4 = cb.next();
        assert!(e4 == None);
        let e5 = cb.next();
        assert!(e5 == None);
    }

    #[test]
    fn test_multiple_context_buffers() {
        let lines: Vec<String> = vec![
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
        ];
        let iter = lines.iter().map(|i| i.to_owned());
        let line_buf = LineBuffer::new(iter);

        // this would fail to compile:
        //let mut cb = ContextBuffer::new(None, line_buf);
        //let mut cb2 = ContextBuffer::new(None, line_buf);

        // but this does compile
        let mb = line_buf;
        let cb = ContextBuffer::new(None, mb);

        let mb2 = cb.into_line_buffer();
        let _ = ContextBuffer::new(None, mb2);
    }
}
