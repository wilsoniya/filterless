use std::collections::{Bound, BTreeSet};
use std::io::BufRead;
use std::io::Lines;
use std::io::{stderr, Write};
use std::cmp::max;

/// A 2-tuple of line number and content string
pub type Line = (usize, String);

pub struct BufferedFilter<B> {
    /// raw buffered lines
    raw_lines: Vec<Line>,
    /// string to filter on
    filter_string: Option<String>,
    /// indices into `raw_lines` where `filter_string` is present
    filter_line_indices: BTreeSet<usize>,
    /// linewise input iterator
    line_iter: Lines<B>,
    /// line number of line displayed at top of last `get_lines()` call
    cur_line: usize,
}

impl<B: BufRead> BufferedFilter<B> {
    pub fn new(lines: Lines<B>) -> BufferedFilter<B> {
        BufferedFilter {
            raw_lines: Vec::new(),
            filter_string: None,
            filter_line_indices: BTreeSet::new(),
            line_iter: lines,
            cur_line: 0,
        }
    }

    pub fn set_filter(&mut self, filter_string: String) {
        match self.filter_string {
            Some(ref _filter_string) => {
                if *_filter_string != filter_string {
                    self.filter_line_indices = BTreeSet::new();
                    self.cur_line = 0;
                }
            },
            None => (),
        };

        self.filter_string = match filter_string.len() {
            0 => None,
            _ => Some(filter_string),
        };
    }

    pub fn get_filter(&self) -> Option<String> {
        self.filter_string.clone()
    }

    pub fn get_buffer_length(&self) -> usize {
        self.raw_lines.len()
    }

    pub fn offset_to_lines(&mut self, offset: i64, num_lines: usize)
        -> Vec<Line> {
        let start_line = max(self.cur_line as i64 + offset, 0) as usize;

        match self.filter_string {
            Some(_) => {
                self.ensure_index_length(start_line, num_lines);
                let first_idx: usize;
                {
                    let idx = self.filter_line_indices
                        .range(Bound::Unbounded, Bound::Unbounded)
                        .nth(start_line);
                    first_idx = match idx {
                        Some(_idx) => {
                            self.cur_line = start_line;
                            *_idx
                        },
                        None => {
                            match self.filter_line_indices.iter().last() {
                                Some(_idx) => {
                                    self.cur_line = self.filter_line_indices.len();
                                    *_idx
                                },
                                None => 0,
                            }
                        },
                    }
                }

                self.ensure_index_length(first_idx, num_lines);
                let ret = self.get_filtered_lines(first_idx, num_lines);

                ret
            }
            None => {
                let buf_len = start_line + num_lines;
                self.ensure_buffer_length(buf_len as usize);
                self.get_unfiltered_lines(start_line, num_lines)
            }
        }
    }

    pub fn ensure_buffer_length(&mut self, num_lines: usize) {
        if num_lines < self.raw_lines.len() {
            // case: buffer already has enough lines
            return;
        }

        let mut total_lines = self.raw_lines.len();
        let remaining_lines = num_lines - total_lines;
        let mut new_lines: Vec<Line> = (&mut self.line_iter)
            .take(remaining_lines)
            .map(|line| {
                let ret = (total_lines, line.unwrap());
                total_lines += 1;
                ret
            })
            .collect();
        self.raw_lines.append(&mut new_lines);
    }

    fn get_unfiltered_lines(&mut self, start_line_num: usize, num_lines: usize)
        -> Vec<Line> {
        self.ensure_buffer_length(start_line_num + num_lines);
        self.cur_line = start_line_num;
        (&self.raw_lines[start_line_num..])
            .iter()
            .map(ToOwned::to_owned)
            .take(num_lines)
            .collect()
    }

    fn ensure_index_length(&mut self, start_line_num: usize,
                           additional_matches: usize) {
        if self.filter_string.is_none() {
            panic!("ensure_index_length() called when filter_string is None");
        }
        let remaining_idx_size = self.filter_line_indices
            .range(Bound::Included(&start_line_num), Bound::Unbounded)
            .count();

        if remaining_idx_size < additional_matches {
            let additional_matches = additional_matches - remaining_idx_size;
            self.expand_filter(additional_matches);
        }
    }

    fn expand_filter(&mut self, additional_matches: usize) {
        if self.filter_string.is_none() {
            return;
        }

        let filter_string: &str = self.filter_string.as_ref().unwrap();

        // determine index of last filter match
        let last_idx = match self.filter_line_indices.iter().last() {
            Some(idx) => *idx,
            None => 0,
        };

        let mut cur_idx = match last_idx {
            0 => 0,
            _ => last_idx + 1,
        };

        let mut found_matches: usize = 0;

        // search buffered lines for filter match
        for line in &self.raw_lines[cur_idx..] {
            if line.1.contains(filter_string) {
                self.filter_line_indices.insert(cur_idx);
                found_matches += 1;
            }

            cur_idx += 1;


            if found_matches >= additional_matches {
                break;
            }

        }

        // scan line iterator for more matches
        if found_matches < additional_matches {
            for line in &mut self.line_iter {

                let line: &str = line.as_ref().unwrap();

                if line.contains(filter_string) {
                    self.filter_line_indices.insert(cur_idx);
                    found_matches += 1;
                }

                let raw_lines_len = self.raw_lines.len();
                self.raw_lines.push((raw_lines_len, line.to_owned()));

                cur_idx += 1;

                if found_matches >= additional_matches {
                    break;
                }
            }
        }
    }

    fn get_filtered_lines(&mut self, start_line_num: usize, num_lines: usize)
        -> Vec<Line> {
        if self.filter_string.is_none() {
            panic!("get_filtered_lines() called when filter_string is None");
        }

        self.ensure_index_length(start_line_num, num_lines);

        self.filter_line_indices
            .range(Bound::Included(&start_line_num), Bound::Unbounded)
            .map(|idx| self.raw_lines.get(*idx))
            .take(num_lines)
            .take_while(|line| line.is_some())
            .map(|line| line.unwrap().clone())
            .collect()
    }
}

fn debug(msg: String) {
    stderr().write(&msg.as_bytes());
    stderr().write("\n".as_bytes());
}
