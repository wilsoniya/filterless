use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Result};
use std::iter::{Iterator, repeat};
use std::fmt;

//static FNAME: &'static str = "/home/wilsoniya/devel/filterless/test";
static FNAME: &'static str = "/home/wilsoniya/devel/filterless/pg730.txt";

/// Parameters used when creating a filtering iterator
#[derive(Clone)]
pub struct FilterPredicate {
    /// Search string which must be included in a line to be considered a match
    pub filter_string: String,
    /// Number of non-match lines above and below a match line to include in
    /// the lines returned by the iterator
    pub context_lines: usize ,
}

pub type NumberedLine = (usize, String);

/// Representation of a line that might be returned from a filtering iterator.
#[derive(Clone, Debug, PartialEq)]
pub enum FilteredLine {
    /// a gap between context groups (i.e., groups of context lines
    /// corresponding to distinct match lines)
    Gap,
    /// a line which provides context before or after a matched line
    ContextLine(NumberedLine),
    /// a line matched by a filter string
    MatchLine(NumberedLine),
    /// a line emitted when no filter predicate is in use
    UnfilteredLine(NumberedLine),
}

#[derive(Clone, Debug)]
/// Representation of a line returned from a ContextBuffer.
enum ContextLine {
    /// the line matched a given filter string
    Match(NumberedLine),
    /// the line did not match the filter string
    NoMatch(NumberedLine),
}

/// Representation of an iterator's encounter with a context gap.
enum Gap {
    /// When the current value in the iterator would produce a context gap
    Current,
    /// When the previous value in the iterator produced a context gap
    Previous,
    /// When a context gap has not been produced in the past two iterations
    None,
}

impl ContextLine {
    /// Creates a `ContextLine` instance by consuming a `NumberedLine`.
    fn from_numbered_line(numbered_line: NumberedLine, filter_string: &String) -> ContextLine {
        if numbered_line.1.contains(filter_string) {
            ContextLine::Match(numbered_line)
        } else {
            ContextLine::NoMatch(numbered_line)
        }
    }

    /// Creates a `FilteredLine` by cloning the inner `NumberedLine`.
    fn to_filtered_line(&self, pred: &Option<FilterPredicate>) -> FilteredLine {
        match self {
            &ContextLine::Match(ref numbered_line) => {
                FilteredLine::MatchLine(numbered_line.to_owned())
            },
            &ContextLine::NoMatch(ref numbered_line) => {
                match pred {
                    &Some(_) => FilteredLine::ContextLine(numbered_line.to_owned()),
                    &None => FilteredLine::UnfilteredLine(numbered_line.to_owned()),
                }
            },
        }
    }

}

impl FilteredLine {
    pub fn get_line_num(&self) -> Option<usize> {
        match self {
            &FilteredLine::Gap => None,
            &FilteredLine::ContextLine((line_num, _)) => Some(line_num),
            &FilteredLine::MatchLine((line_num, _)) => Some(line_num),
            &FilteredLine::UnfilteredLine((line_num, _)) => Some(line_num),
        }
    }
}

/// Thing which reads, caches, and makes filterable lines produced by linewise
/// iterators.
pub struct FilteringLineBuffer<B: BufRead> {
    lines: Lines<B>,
    cached_lines: Vec<(usize, String)>,
}

impl<B: BufRead> FilteringLineBuffer<B> {
    /// Creates a new `FilteringLineBuffer` from a linewise iterator.
    pub fn new(buf: B) -> FilteringLineBuffer<B> {
        FilteringLineBuffer {
            lines: buf.lines(),
            cached_lines: Vec::new(),
        }
    }

    /// Creates a filtered iterator over the line buffer.
    ///
    /// ### Parameters
    /// * `offset`: number of lines from the beginning of the raw lines to
    ///   start considering lines to filter
    /// * `filter`: parameters on which to base resultant iterator's behavior
    pub fn iter(&mut self, offset: usize, filter:
                Option<FilterPredicate>) -> ContextBuffer<Self> {
//      FilteringLineIter::new(self, offset, filter)
        ContextBuffer::new(filter, self)
    }

    /// Gets a copy of the `line_num`th line as read off the input lines.
    ///
    /// ### Parameters
    /// * `line_num`: 1-indexed index of the line of the underlying buffer to
    ///   return
    pub fn get(&mut self, line_num: usize) -> Option<NumberedLine> {
        if line_num < 1 {
            // case: reject non-1-indexed indexes
            return None;
        }

        let cache_idx = line_num - 1;
        let last_line_num = self.cached_lines.len();

        if line_num > last_line_num {
            // case: not enough lines in cache; load more from line iter
            let _ = self
                .take_while(|&(i, _)| line_num > i)
                .collect::<Vec<(usize, String)>>();
        }

        self.cached_lines.get(cache_idx).map(|i| i.to_owned())
    }
}

impl<B: BufRead> Iterator for FilteringLineBuffer<B> {
    type Item = NumberedLine;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().map(|line| {
            // TODO: what happens when lines are read as Err()?
            let line_num = self.cached_lines.len() + 1;
            let line = line.expect(
                &format!("Can't read line number {}", line_num));
            let line_copy = line.clone();
            self.cached_lines.push((line_num, line));
            (line_num, line_copy)
        })
    }
}

// pub struct FilteringLineIter<'a, B: 'a + BufRead> {
//     buffer: &'a mut FilteringLineBuffer<B>,
//     /// num lines from top of input lines to start iterating
//     offset: usize,
//     /// criteria on which lines are filtered, if any
//     filter: Option<FilterPredicate>,
//     /// last line fetched from underlying buffer
//     last_line: usize,
//     /// `true` when underlying buffer is exhausted
//     buffer_exhausted: bool,
// }
//
// impl<'a, B: BufRead + 'a> FilteringLineIter<'a, B> {
//     pub fn new(buffer: &'a mut FilteringLineBuffer<B>, offset: usize,
//            filter: Option<FilterPredicate>) -> FilteringLineIter<'a, B> {
//         let last_line = offset;
//
//         FilteringLineIter {
//             buffer: buffer,
//             offset: offset,
//             filter: filter,
//             last_line: last_line,
//             buffer_exhausted: false,
//         }
//     }
//
//     fn next_unfiltered(&mut self) -> Option<FilteredLine> {
//         let line_num = self.last_line + 1;
//
//         self.buffer.get(line_num)
//             .map(|line| {
//                 self.last_line = line_num;
//                 FilteredLine::UnfilteredLine(line)
//             })
//     }
//
//     fn next_filtered_without_context(
//         &mut self, pred: &FilterPredicate) -> Option<FilteredLine> {
//         let cur_line_num = self.last_line + 1;
//
//         let ret = (cur_line_num..)
//             .map(|i| self.buffer.get(i))
//             .take_while(|maybe_line| maybe_line.is_some())
//             .filter_map(|maybe_line| maybe_line)
//             .skip_while(|&(_, ref line)| !line.contains(&pred.filter_string))
//             .map(|line| {
//                 FilteredLine::MatchLine(line)
//             })
//         .nth(0);
//
//         self.last_line = ret
//             .as_ref()
//             .and_then(|filtered_line| filtered_line.get_line_num())
//             .unwrap_or(self.last_line);
//
//         ret
//     }
// }
//
// impl<'a, B: BufRead + 'a> Iterator for FilteringLineIter<'a, B> {
//     type Item = FilteredLine;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buffer_exhausted {
//             None
//         } else {
//             let line_num = self.last_line + 1;
//
//             // clone because closure closes over self to access pred
//             let filter_copy = self.filter.clone();
//             let ret = filter_copy.map(|pred| {
//                 // case: active filter predicate; filter lines
//                 self.next_filtered_without_context(&pred)
//             })
//             .unwrap_or_else(|| {
//                 // case: no active filter predicate; emit all lines
//                 self.next_unfiltered()
//             });
//
//             self.buffer_exhausted = ret.is_none();
//             ret
//         }
//     }
// }

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
pub struct ContextBuffer<'a, T: Iterator + 'a> {
    filter_predicate: Option<FilterPredicate>,
    /// earlier lines in lower indexes
    buffer: VecDeque<Option<ContextLine>>,
    /// underlying iterator
    iter: &'a mut T,
    gap: Gap
}

impl<'a, T: Iterator<Item = NumberedLine> + 'a> ContextBuffer<'a, T> {
    fn new(filter_predicate: Option<FilterPredicate>,
           iter: &'a mut T) -> ContextBuffer<'a, T> {


        let buffer = match filter_predicate {
            Some(FilterPredicate{ ref filter_string, ref context_lines }) => {
                let capacity = context_lines * 2 + 1;
                repeat(None)
                    .take(context_lines + 1)
                    .chain(iter.map(|numbered_line| {
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
            Some(FilterPredicate{ ref filter_string, ref context_lines }) => {
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
            Some(FilterPredicate{ ref filter_string, ref context_lines }) => {
                let matches = self.buffer.iter().map(|maybe_line| {
                    match maybe_line {
                        &Some(ContextLine::Match(_)) => true,
                        _ => false,
                    }
                }).collect::<Vec<bool>>();

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
}

impl<'a, T: Iterator<Item = NumberedLine> + 'a> Iterator for ContextBuffer<'a, T> {
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

fn main() {
    let file = File::open(FNAME).unwrap();
    let reader = BufReader::new(file);

    let mut buffer = FilteringLineBuffer::new(reader);
    let pred = FilterPredicate { filter_string: "OLIVER".to_owned(), context_lines: 1 };
    let iter = buffer.iter(0, Some(pred));

    println!("{:?}", iter.collect::<Vec<FilteredLine>>());
}

#[cfg(test)]
mod test {
    use ::ContextBuffer;
    use ::FilteredLine;
    use ::FilterPredicate;

    #[test]
    fn test1() {
        let mut lines: Vec<(usize, String)> = vec![
            (0, "none".to_owned()),
            (1, "ctx".to_owned()),
            (2, "ctx".to_owned()),
            (3, "match".to_owned()),
            (4, "ctx".to_owned()),
            (5, "ctx".to_owned()),
            (6, "none".to_owned()),
            (7, "none".to_owned()),
            (8, "ctx".to_owned()),
            (9, "ctx".to_owned()),
            (10, "match".to_owned()),
            (11, "ctx".to_owned()),
        ];
        let context_lines = 2;
        let filter_string = "match".to_owned();
        let mut iter = lines.iter();

        let pred = FilterPredicate {
            filter_string: filter_string,
            context_lines: context_lines
        };
        let mut cb = ContextBuffer::new(Some(pred), &mut iter);

        let e0 = cb.next();
        assert!(e0 == Some(FilteredLine::Gap));
        let e1 = cb.next();
        assert!(e1 == Some(FilteredLine::ContextLine((1, String::from("ctx")))));
        let e2 = cb.next();
        assert!(e2 == Some(FilteredLine::ContextLine((2, String::from("ctx")))));
        let e3 = cb.next();
        assert!(e3 == Some(FilteredLine::MatchLine((3, String::from("match")))));
        let e4 = cb.next();
        assert!(e4 == Some(FilteredLine::ContextLine((4, String::from("ctx")))));
        let e5 = cb.next();
        assert!(e5 == Some(FilteredLine::ContextLine((5, String::from("ctx")))));
        let e6 = cb.next();
        assert!(e6 == Some(FilteredLine::Gap));
        let e7 = cb.next();
        assert!(e7 == Some(FilteredLine::ContextLine((8, String::from("ctx")))));
        let e8 = cb.next();
        assert!(e8 == Some(FilteredLine::ContextLine((9, String::from("ctx")))));
        let e9 = cb.next();
        assert!(e9 == Some(FilteredLine::MatchLine((10, String::from("match")))));
        let e10 = cb.next();
        assert!(e10 == Some(FilteredLine::ContextLine((11, String::from("ctx")))));
    }

    #[test]
    fn test2() {
        let mut lines: Vec<(usize, String)> = vec![
            (0, "match".to_owned()),
            (1, "match".to_owned()),
            (2, "none".to_owned()),
            (3, "match".to_owned()),
            (4, "none".to_owned()),
        ];
        let context_lines = 0;
        let filter_string = "match".to_owned();
        let mut iter = lines.iter();

        let pred = FilterPredicate {
            filter_string: filter_string,
            context_lines: context_lines
        };
        let mut cb = ContextBuffer::new(Some(pred), &mut iter);

        let e0 = cb.next();
        println!("{:?}", e0);
        assert!(e0 == Some(FilteredLine::MatchLine((0, String::from("match")))));
        let e1 = cb.next();
        assert!(e1 == Some(FilteredLine::MatchLine((1, String::from("match")))));
        let e2 = cb.next();
        assert!(e2 == Some(FilteredLine::Gap));
        let e3 = cb.next();
        assert!(e3 == Some(FilteredLine::MatchLine((3, String::from("match")))));
        let e4 = cb.next();
        assert!(e4 == None);
    }

    #[test]
    fn test3() {
        let mut lines: Vec<(usize, String)> = vec![
            (1, "one".to_owned()),
            (2, "two".to_owned()),
            (3, "three".to_owned()),
        ];
        let mut iter = lines.iter();

        let mut cb = ContextBuffer::new(None, &mut iter);

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
}
