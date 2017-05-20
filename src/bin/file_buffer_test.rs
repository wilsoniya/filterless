use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Result};
use std::iter::Iterator;

static FNAME: &'static str = "/home/wilsoniya/devel/filterless/test";

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
            let line_num = self.cached_lines.len() + 2;
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
    buffer_exhausted: bool
}

impl<'a, B: BufRead + 'a> FilteringLineIter<'a, B> {
    fn new(buffer: &'a mut FilteringLineBuffer<B>, offset: usize,
           filter: Option<FilterPredicate>) -> FilteringLineIter<'a, B> {
        let last_line = offset;

        FilteringLineIter {
            buffer: buffer,
            offset: offset,
            filter: filter,
            last_line: last_line,
            buffer_exhausted: false
        }
    }
}

impl<'a, B: BufRead + 'a> Iterator for FilteringLineIter<'a, B> {
    type Item = FilteredLine;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer_exhausted {
            None
        } else {
            let line_num = self.last_line + 1;

            let filter_copy = self.filter.clone();
            let ret = filter_copy.map(|pred| {
                // case: active filter predicate; filter lines
                ((line_num + 1)..)
                    .map(|i| self.buffer.get(i))
                    .filter_map(|i| i)
                    .skip_while(|&(_, ref line)| !line.contains(&pred.filter_string))
                    .map(|line| FilteredLine::MatchLine(line))
                    .nth(0)
            })
            .unwrap_or_else(|| {
                // case: no active filter predicate; emit all lines
                self.buffer.get(line_num)
                    .map(|line| {
                        self.last_line = line_num;
                        FilteredLine::UnfilteredLine(line)
                    })
            });

            self.buffer_exhausted = ret.is_none();
            ret
        }
    }
}

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

fn main() {
    let file = File::open(FNAME).unwrap();
    let reader = BufReader::new(file);

    let mut buffer = FilteringLineBuffer::new(reader);
    let iter = buffer.iter(0, None);
}

fn get_file() -> Result<File> {
    let file = File::open(FNAME)?;

    Ok(file)
}

