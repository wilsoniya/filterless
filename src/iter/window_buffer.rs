use std::io::BufRead;

use super::line_buffer::LineBuffer;
use super::context_buffer::ContextBuffer;
use super::iter;

/// Thing which filters, describes, and categorizes lines from an iterator
/// according to some specific filtering criteria.
pub struct WindowBuffer<T: Iterator<Item=String>> {
    /// line source and filtering apparatus
    context_buffer: Option<ContextBuffer<T>>,
    /// cache of lines that have been read off of `context_buffer`
    buffered_lines: Vec<iter::FilteredLine>,
    /// criteria on which lines are filtered by `context_buffer`
    predicate: Option<iter::FilterPredicate>,
    /// width of window in columns
    width: usize,
    /// height of window in lines
    height: usize,
    /// 1-indexed offset of line at top of window
    start_line: usize,
    /// 1-offset index of line at bottom of window
    end_line: usize,
}

impl<T: Iterator<Item=String>> WindowBuffer<T> {
    /// Creates a new `WindowBuffer`.
    ///
    /// ### Parameters
    /// * `predicate`: optinal filtering criteria applied to underlying line
    ///   source
    /// * `width`: width of the terminal window in columns
    /// * `height`: height of the terminal window in rows
    pub fn new(mut iter: T,
           predicate: Option<iter::FilterPredicate>,
           width: usize,
           height: usize) -> Self {

        let line_buffer = LineBuffer::new(iter);
        let context_buffer = Some(ContextBuffer::new(predicate.clone(), line_buffer));

        let mut ret = WindowBuffer {
            context_buffer: context_buffer,
            buffered_lines: Vec::new(),
            predicate: predicate,
            width: width,
            height: height,
            start_line: 0,
            end_line: 0,
        };

        ret
    }

    /// Sets the filter predicate.
    ///
    /// This also has the effect of purging the buffer and setting the current
    /// position to zero.
    pub fn set_predicate(&mut self, predicate: Option<iter::FilterPredicate>) {
        let mut context_buffer = self.context_buffer
            .take()
            .expect("context_buffer must always be Some");

        let mut line_buffer = context_buffer.into_line_buffer();
        line_buffer.seek(Some(1), None);
        self.context_buffer = Some(ContextBuffer::new(predicate.clone(), line_buffer));
        self.predicate = predicate;
        self.buffered_lines.clear();

        // XXX it's probably not desireable to reset the line number to zero
        // when the filter predicate is changed
        self.end_line = 0;
    }

    /// Gets the next line after the line currently displayed at the bottom of
    /// the window.
    pub fn next_line(&mut self) -> Option<iter::FilteredLine> {
        let next_line = self.end_line + 1;

        let lines = self.get_lines(next_line, 1);
        self.end_line = if lines.len() > 0 { next_line } else { self.end_line };

        lines.first().map(|line| line.to_owned())
    }

    /// Gets the line before the line at the top of the window.
    pub fn prev_line(&mut self) -> Option<iter::FilteredLine> {
        if self.end_line as i64 - self.height as i64 <= 0 {
            // case already at the beginning; can't go back farther
            return None
        }

        let next_line = if self.end_line - self.height > 0 { self.end_line - self.height } else { 0 };
        let new_end_line = if self.end_line > 0 { self.end_line - 1 } else { 0 };

        let lines = self.get_lines(next_line, 1);
        self.end_line = if lines.len() > 0 { new_end_line } else { self.end_line };

        lines.first().map(|line| line.to_owned())
    }

    /// Gets a page full of lines beginning after the line currently displayed
    /// at the bottom of the window.
    pub fn next_page(&mut self) -> Vec<iter::FilteredLine> {
        let start_line = self.end_line + 1;
        let num_lines = self.height;
        let lines = self.get_lines(start_line, num_lines);

        lines
    }

    /// Gets a page full of lines ending before the line currently displayed
    /// at the top of the window.
    pub fn prev_page(&mut self) -> Vec<iter::FilteredLine> {
        let start_line = if (self.start_line as i64 - self.height as i64 >= 1) {
            self.start_line - self.height
        } else {
            1
        };

        let num_lines = self.height;
        let lines = self.get_lines(start_line, num_lines);

        lines
    }

    /// Gets lines in range.
    ///
    /// ### Parameters
    /// * `start`: 1-based index of the first line to return
    /// * `num_lines`: number of lines to return
    fn get_lines(&mut self, start: usize, num_lines: usize) -> Vec<iter::FilteredLine> {
        assert!(start >= 1, format!("first line number must be at least 1; got {}", start));
        let start = start - 1;
        let end_desired = start + num_lines;
        self.fill_buffer(end_desired);

        let end = if end_desired <= self.buffered_lines.len() {
            end_desired
        } else {
            self.buffered_lines.len()
        };


        self.start_line = start + 1;
        self.end_line = end;

        if let Some(lines) = self.buffered_lines.get(start..end) {
            lines.to_owned()
        } else {
            Vec::new()
        }
    }

    fn fill_buffer(&mut self, limit: usize) {
        let num_new_lines = limit as i64 - self.buffered_lines.len() as i64;

        let context_buffer = self.context_buffer
            .as_mut()
            .expect("context_buffer must always be Some");

        if num_new_lines > 0 {
            let new_lines = context_buffer.take(num_new_lines as usize);
            self.buffered_lines.extend(new_lines);
        }
    }
}

mod test {
    use super::{WindowBuffer};
    use iter::iter::{NumberedLine, FilteredLine, FilterPredicate};

    #[test]
    fn test_prev_next() {
        let vec: Vec<String> = vec!(
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
            "four".to_owned(),
            "five".to_owned(),
            "six".to_owned(),
            "seven".to_owned(),
            "eight".to_owned(),
            "nine".to_owned(),
            "ten".to_owned(),
        );

        let iter = vec.iter().map(|i| i.to_owned());

        let mut obj_ut = WindowBuffer::new(iter, None, 80, 3);

        assert_eq!(obj_ut.prev_line(), None);

        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((1, "one".to_owned()))));
        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((2, "two".to_owned()))));
        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((3, "three".to_owned()))));
        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((4, "four".to_owned()))));
        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((5, "five".to_owned()))));
        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((6, "six".to_owned()))));
        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((7, "seven".to_owned()))));
        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((8, "eight".to_owned()))));
        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((9, "nine".to_owned()))));
        assert_eq!(obj_ut.next_line()
               , Some(FilteredLine::UnfilteredLine((10, "ten".to_owned()))));
        assert_eq!(obj_ut.next_line(), None);

        assert_eq!(obj_ut.prev_line()
               , Some(FilteredLine::UnfilteredLine((7, "seven".to_owned()))));
        assert_eq!(obj_ut.prev_line()
               , Some(FilteredLine::UnfilteredLine((6, "six".to_owned()))));
        assert_eq!(obj_ut.prev_line()
               , Some(FilteredLine::UnfilteredLine((5, "five".to_owned()))));
        assert_eq!(obj_ut.prev_line()
               , Some(FilteredLine::UnfilteredLine((4, "four".to_owned()))));
        assert_eq!(obj_ut.prev_line()
               , Some(FilteredLine::UnfilteredLine((3, "three".to_owned()))));
        assert_eq!(obj_ut.prev_line()
               , Some(FilteredLine::UnfilteredLine((2, "two".to_owned()))));
        assert_eq!(obj_ut.prev_line()
               , Some(FilteredLine::UnfilteredLine((1, "one".to_owned()))));
        assert_eq!(obj_ut.prev_line(), None);
    }

    #[test]
    fn test_paging() {
        let vec: Vec<String> = vec!(
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
            "four".to_owned(),
            "five".to_owned(),
            "six".to_owned(),
            "seven".to_owned(),
            "eight".to_owned(),
            "nine".to_owned(),
            "ten".to_owned(),
        );
        let iter = vec.iter().map(|i| i.to_owned());

        let mut obj_ut = WindowBuffer::new(iter, None, 80, 3);

        assert_eq!(obj_ut.prev_page(), vec![
                   FilteredLine::UnfilteredLine((1, "one".to_owned())),
                   FilteredLine::UnfilteredLine((2, "two".to_owned())),
                   FilteredLine::UnfilteredLine((3, "three".to_owned())),
        ]);
        assert_eq!(obj_ut.next_page(), vec![
                   FilteredLine::UnfilteredLine((4, "four".to_owned())),
                   FilteredLine::UnfilteredLine((5, "five".to_owned())),
                   FilteredLine::UnfilteredLine((6, "six".to_owned())),
        ]);
        assert_eq!(obj_ut.next_page(), vec![
                   FilteredLine::UnfilteredLine((7, "seven".to_owned())),
                   FilteredLine::UnfilteredLine((8, "eight".to_owned())),
                   FilteredLine::UnfilteredLine((9, "nine".to_owned())),
        ]);
        assert_eq!(obj_ut.next_page(), vec![
                   FilteredLine::UnfilteredLine((10, "ten".to_owned())),
        ]);
        assert_eq!(obj_ut.next_page(), Vec::new());
        assert_eq!(obj_ut.next_page(), Vec::new());

        assert_eq!(obj_ut.prev_page(), vec![
                   FilteredLine::UnfilteredLine((8, "eight".to_owned())),
                   FilteredLine::UnfilteredLine((9, "nine".to_owned())),
                   FilteredLine::UnfilteredLine((10, "ten".to_owned())),
        ]);

        assert_eq!(obj_ut.prev_page(), vec![
                   FilteredLine::UnfilteredLine((5, "five".to_owned())),
                   FilteredLine::UnfilteredLine((6, "six".to_owned())),
                   FilteredLine::UnfilteredLine((7, "seven".to_owned())),
        ]);

        assert_eq!(obj_ut.prev_page(), vec![
                   FilteredLine::UnfilteredLine((2, "two".to_owned())),
                   FilteredLine::UnfilteredLine((3, "three".to_owned())),
                   FilteredLine::UnfilteredLine((4, "four".to_owned())),
        ]);

        assert_eq!(obj_ut.prev_page(), vec![
                   FilteredLine::UnfilteredLine((1, "one".to_owned())),
                   FilteredLine::UnfilteredLine((2, "two".to_owned())),
                   FilteredLine::UnfilteredLine((3, "three".to_owned())),
        ]);

        assert_eq!(obj_ut.prev_page(), vec![
                   FilteredLine::UnfilteredLine((1, "one".to_owned())),
                   FilteredLine::UnfilteredLine((2, "two".to_owned())),
                   FilteredLine::UnfilteredLine((3, "three".to_owned())),
        ]);

        assert_eq!(obj_ut.next_page(), vec![
                   FilteredLine::UnfilteredLine((4, "four".to_owned())),
                   FilteredLine::UnfilteredLine((5, "five".to_owned())),
                   FilteredLine::UnfilteredLine((6, "six".to_owned())),
        ]);
    }

    #[test]
    fn test_predicate() {
        let vec: Vec<String> = vec!(
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
            "four".to_owned(),
            "five".to_owned(),
            "six".to_owned(),
            "seven".to_owned(),
            "eight".to_owned(),
            "nine".to_owned(),
            "ten".to_owned(),
        );
        let iter = vec.iter().map(|i| i.to_owned());

        let mut predicate = Some(FilterPredicate{
            filter_string: "t".to_owned(),
            context_lines: 0,
        });
        let mut obj_ut = WindowBuffer::new(iter, predicate, 80, 3);

        assert_eq!(obj_ut.next_line(), Some(FilteredLine::Gap));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::MatchLine((2, "two".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::MatchLine((3, "three".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::Gap));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::MatchLine((8, "eight".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::Gap));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::MatchLine((10, "ten".to_owned()))));
        assert_eq!(obj_ut.next_line(), None);

        predicate = Some(FilterPredicate{
            filter_string: "t".to_owned(),
            context_lines: 1,
        });
        obj_ut.set_predicate(predicate);

        assert_eq!(obj_ut.next_line(), Some(FilteredLine::ContextLine((1, "one".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::MatchLine((2, "two".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::MatchLine((3, "three".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::ContextLine((4, "four".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::Gap));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::ContextLine((7, "seven".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::MatchLine((8, "eight".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::ContextLine((9, "nine".to_owned()))));
        assert_eq!(obj_ut.next_line(), Some(FilteredLine::MatchLine((10, "ten".to_owned()))));
        assert_eq!(obj_ut.next_line(), None);
    }
}
