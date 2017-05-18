use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Result};
use std::iter::Iterator;

static fname: &'static str = "/home/wilsoniya/devel/filterless/test";

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

    /// Gets the `line_num`th line as read off the input lines.
    pub fn get(&mut self, line_num: usize) -> Option<String> {
        let mut last_line_num = self.cached_lines.len();

        if line_num > last_line_num {
            // case: not enough lines in cache; load more from line iter
            let mut new_lines = ((last_line_num + 1)..)
                .zip((&mut self.lines))
                .take_while(|&(i, _)| line_num > i)
                // TODO: what happens when lines are read as Err()?
                .map(|(i, maybe_line)| (i, maybe_line.unwrap()))
                .collect::<Vec<(usize, String)>>();

            self.cached_lines.append(&mut new_lines);
        }

        self.cached_lines.get(line_num).map(|&(_, ref line)| line.to_owned())
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
        FilteringLineIter {
            buffer: buffer,
            offset: offset,
            filter: filter,
            last_line: 0,
            buffer_exhausted: false
        }
    }
}

impl<'a, B: BufRead + 'a> Iterator for FilteringLineIter<'a, B> {
    type Item = (usize, FilteredLine);

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer_exhausted {
            None
        } else {
            let line_num = self.last_line + 1;
            match self.buffer.get(line_num) {
                Some(line) => {
                    self.last_line = line_num;
                    Some((line_num, FilteredLine::MatchLine(line)))
                },
                None => None
            }
        }
    }
}

/// Parameters used when creating a filtering iterator
pub struct FilterPredicate {
    /// Search string which must be included in a line to be considered a match
    pub filter_string: String,
    /// Number of non-match lines above and below a match line to include in
    /// the lines returned by the iterator
    pub context_lines: usize ,
}

pub enum FilteredLine {
    Gap,
    ContextLine(String),
    MatchLine(String),
}

//impl<B: BufRead> Iterator for FilteringLineBuffer<B> {
//    type Item = (usize, String);
//
//    fn next(&mut self) -> Option<Self::Item> {
//        self.cached_lines.last().map(&(ref l, _) l + 1)
//    }
//}

//fn main() {
//    let file = File::open(fname).unwrap();
//    let reader = BufReader::new(file);
//    let lines = reader.lines();
//    let idx_lines: Vec<(usize, String)> = lines
//        .enumerate()
//        .map(|(n, l)| (n+1, l.ok()))
//        .filter(|&(ref n, ref l)| l.is_some())
//        .map(|(n, l)| (n, l.unwrap()))
//        .collect();
//
//    println!("{:?}", idx_lines);
//}

fn main() {
    let file = File::open(fname).unwrap();
    let reader = BufReader::new(file);

    let mut buffer = FilteringLineBuffer::new(reader);
    let iter = buffer.iter(0, None);
}

fn get_file() -> Result<File> {
    let file = File::open(fname)?;

    Ok(file)
}

