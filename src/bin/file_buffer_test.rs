use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Result};
use std::iter::{Iterator, repeat};

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
#[derive(Clone, Debug)]
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

#[derive(Clone)]
enum ContextLine {
    Match(NumberedLine),
    NoMatch(NumberedLine),
}

impl ContextLine {
    fn from_numbered_line(numbered_line: NumberedLine, filter_string: &String) -> ContextLine {
        if numbered_line.1.contains(filter_string) {
            ContextLine::Match(numbered_line)
        } else {
            ContextLine::NoMatch(numbered_line)
        }
    }

    fn get_line(&self) -> NumberedLine {
        match self {
            &ContextLine::Match(ref numbered_line) => numbered_line.to_owned(),
            &ContextLine::NoMatch(ref numbered_line) => numbered_line.to_owned(),
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
                Option<FilterPredicate>) -> FilteringLineIter<B> {
        FilteringLineIter::new(self, offset, filter)
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
            let line = line.unwrap();
            let line_copy = line.clone();
            let line_num = self.cached_lines.len() + 1;
            self.cached_lines.push((line_num, line));
            (line_num, line_copy)
        })
    }
}

pub struct FilteringLineIter<'a, B: 'a + BufRead> {
    buffer: &'a mut FilteringLineBuffer<B>,
    /// num lines from top of input lines to start iterating
    offset: usize,
    /// criteria on which lines are filtered, if any
    filter: Option<FilterPredicate>,
    /// last line fetched from underlying buffer
    last_line: usize,
    /// `true` when underlying buffer is exhausted
    buffer_exhausted: bool,
    /// elements buffered
    context_buffer: VecDeque<Option<NumberedLine>>,
}

impl<'a, B: BufRead + 'a> FilteringLineIter<'a, B> {
    pub fn new(buffer: &'a mut FilteringLineBuffer<B>, offset: usize,
           filter: Option<FilterPredicate>) -> FilteringLineIter<'a, B> {
        let last_line = offset;

        FilteringLineIter {
            buffer: buffer,
            offset: offset,
            filter: filter,
            last_line: last_line,
            buffer_exhausted: false,
            context_buffer: VecDeque::new(),
        }
    }

    fn next_unfiltered(&mut self) -> Option<FilteredLine> {
        let line_num = self.last_line + 1;

        self.buffer.get(line_num)
            .map(|line| {
                self.last_line = line_num;
                FilteredLine::UnfilteredLine(line)
            })
    }

    fn next_filtered_without_context(
        &mut self, pred: &FilterPredicate) -> Option<FilteredLine> {
        let cur_line_num = self.last_line + 1;

        let ret = (cur_line_num..)
            .map(|i| self.buffer.get(i))
            .take_while(|maybe_line| maybe_line.is_some())
            .filter_map(|maybe_line| maybe_line)
            .skip_while(|&(_, ref line)| !line.contains(&pred.filter_string))
            .map(|line| {
                FilteredLine::MatchLine(line)
            })
        .nth(0);

        self.last_line = ret
            .as_ref()
            .and_then(|filtered_line| filtered_line.get_line_num())
            .unwrap_or(self.last_line);

        ret
    }
}

impl<'a, B: BufRead + 'a> Iterator for FilteringLineIter<'a, B> {
    type Item = FilteredLine;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer_exhausted {
            None
        } else {
            let line_num = self.last_line + 1;

            // clone because closure closes over self to access pred
            let filter_copy = self.filter.clone();
            let ret = filter_copy.map(|pred| {
                // case: active filter predicate; filter lines
                self.next_filtered_without_context(&pred)
            })
            .unwrap_or_else(|| {
                // case: no active filter predicate; emit all lines
                self.next_unfiltered()
            });

            self.buffer_exhausted = ret.is_none();
            ret
        }
    }
}

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
struct ContextBuffer<'a, T: Iterator + 'a> {
    /// number of lines to display before and after matched lines
    context_lines: usize,
    /// string whose presence in iterator lines indicates a match
    filter_string: String,
    /// earlier lines in lower indexes
    buffer: VecDeque<Option<ContextLine>>,
    /// underlying iterator
    iter: &'a mut T,
}

impl<'a, T: Iterator<Item = NumberedLine> + 'a> ContextBuffer<'a, T> {
    fn new(context_lines: usize, filter_string: String,
           iter: &'a mut T) -> ContextBuffer<'a, T> {
        let initial_contents = context_lines + 1;
        let capacity = context_lines * 2 + 1;
        let buffer = VecDeque::with_capacity(capacity);

        ContextBuffer {
            context_lines: context_lines,
            filter_string: filter_string,
            buffer: buffer,
            iter: iter,
        }
    }

    fn fill_buffer(&mut self) {
        self.buffer.pop_front();

        let capacity = self.context_lines * 2 + 1;
        let num_elts = self.buffer.len();
        let num_add = capacity - num_elts;
        let blanks = if num_elts > 0 { 0 } else { self.context_lines };

        let filter_string = self.filter_string.clone();
        let mut tail_buf = repeat(None)
            .take(blanks)
            .chain(self.iter.map(|numbered_line| {
                Some(ContextLine::from_numbered_line(numbered_line, &filter_string))
            }))
            .chain(repeat(None))
            .take(num_add)
            .collect();

        self.buffer.append(&mut tail_buf);
    }

    fn classify_cur_line(&self) -> Option<FilteredLine> {
        let matches = self.buffer.iter().map(|maybe_line| {
            match maybe_line {
                &Some(ContextLine::Match(_)) => true,
                _ => false,
            }
        }).collect::<Vec<bool>>();

        let cur_idx = self.context_lines;
        self.buffer.get(cur_idx)
            .and_then(|maybe_context_line| {
                maybe_context_line.as_ref().map(|context_line| {
                    let line = context_line.get_line();
                    let is_match = *matches.get(cur_idx).unwrap_or(&false);
                    let is_context = !is_match && matches.iter().any(|m| *m);

                    if is_match {
                        FilteredLine::MatchLine(line)
                    } else if is_context {
                        FilteredLine::ContextLine(line)
                    } else {
                        FilteredLine::Gap
                    }
                })
            })
    }
}

impl<'a, T: Iterator<Item = NumberedLine> + 'a> Iterator for ContextBuffer<'a, T> {
    type Item = FilteredLine;

    fn next(&mut self) -> Option<Self::Item> {
        let cur_idx = self.context_lines;
        self.buffer.get(cur_idx)
            .map(|maybe_line| maybe_line.to_owned())
            .and_then(|maybe_line| {
                maybe_line.map(|line| {
                    FilteredLine::MatchLine(line.get_line())
                })
            })
    }
}

fn main() {
    let file = File::open(FNAME).unwrap();
    let reader = BufReader::new(file);

    let mut buffer = FilteringLineBuffer::new(reader);
    let pred = FilterPredicate { filter_string: "OLIVER".to_owned(), context_lines: 0 };
    let iter = buffer.iter(0, Some(pred));
//  let iter = buffer.iter(0, None);

    println!("{:?}", iter.collect::<Vec<FilteredLine>>());
}
