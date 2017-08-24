use std::collections::VecDeque;
use std::iter::{Iterator, repeat};

use super::line_buffer::LineBuffer;
use super::iter::{ContextLine, FilteredLine, FilterPredicate, Gap, NumberedLine};

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
    fn new(filter_predicate: Option<FilterPredicate>,
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

