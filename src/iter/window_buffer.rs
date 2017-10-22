use std::io::BufRead;

use super::line_buffer::LineBuffer;
use super::context_buffer::ContextBuffer;
use super::iter;

struct WindowBuffer<T: Iterator<Item=String>> {
    context_buffer: Option<ContextBuffer<T>>,
    buffered_lines: Vec<iter::FilteredLine>,
    predicate: Option<iter::FilterPredicate>,
    /// width of window in columns
    width: usize,
    /// height of window in lines
    height: usize,
    /// 1-offset index of line at top of window
    cur_line: usize,
}

impl<T: Iterator<Item=String>> WindowBuffer<T> {
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
            cur_line: 0,
        };

        ret
    }

    pub fn set_predicate(&mut self, predicate: Option<iter::FilterPredicate>) {
        let mut context_buffer = self.context_buffer
            .take()
            .expect("context_buffer must always be Some");

        let mut line_buffer = context_buffer.into_line_buffer();
        line_buffer.seek(Some(1), None);
        self.context_buffer = Some(ContextBuffer::new(predicate.clone(), line_buffer));
        self.predicate = predicate;

        // XXX it's probably not desireable to reset the line number to zero
        // when the filter predicate is changed
        self.cur_line = 0;
    }

    pub fn next_line(&mut self) -> Option<iter::FilteredLine> {
        let next_line = self.cur_line + 1;

        let lines = self.get_lines(next_line, 1);
        self.cur_line = if lines.len() > 0 { next_line } else { self.cur_line };

        lines.first().map(|line| line.to_owned())
    }

    pub fn prev_line(&mut self) -> Option<iter::FilteredLine> {
        if self.cur_line as i64 - 1 <= 0 {
            // case already at the beginning; can't go back farther
            return None
        }

        let next_line = if self.cur_line > 0 { self.cur_line - 1 } else { 0 };

        let lines = self.get_lines(next_line, 1);
        self.cur_line = if lines.len() > 0 { next_line } else { self.cur_line };

        lines.first().map(|line| line.to_owned())
    }

    pub fn next_page(&mut self) -> Vec<iter::FilteredLine> {
        let first_line = self.cur_line + 1;
        let num_lines = self.height;
        println!("next_page(): first_line: {}; num_lines: {}", first_line, num_lines);
        let lines = self.get_lines(first_line, num_lines);

        self.cur_line = if lines.len() > 0 {
            self.cur_line + lines.len()
        } else {
            self.buffered_lines.len() + 1
        };

        lines
    }

    pub fn prev_page(&mut self) -> Vec<iter::FilteredLine> {
        let (first_line, num_lines) = if self.cur_line as i64 - self.height as i64 >= 1 {
            (self.cur_line - self.height, self.height)
        } else {
            let num_lines = if self.cur_line > 0 { self.cur_line - 1 } else { 0 };
            (1, num_lines)
        };

        println!("prev_page(): first_line: {}; num_lines: {}", first_line, num_lines);
        let lines = self.get_lines(first_line, num_lines);

        self.cur_line = if lines.len() > 0 { first_line } else { 0 };

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
    use iter::iter::{NumberedLine, FilteredLine};

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
               , Some(FilteredLine::UnfilteredLine((9, "nine".to_owned()))));
        assert_eq!(obj_ut.prev_line()
               , Some(FilteredLine::UnfilteredLine((8, "eight".to_owned()))));
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

        assert_eq!(obj_ut.prev_page(), Vec::new());
        assert_eq!(obj_ut.next_page(), vec![
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
        ]);

        assert_eq!(obj_ut.prev_page(), Vec::new());

        assert_eq!(obj_ut.next_page(), vec![
                   FilteredLine::UnfilteredLine((1, "one".to_owned())),
                   FilteredLine::UnfilteredLine((2, "two".to_owned())),
                   FilteredLine::UnfilteredLine((3, "three".to_owned())),
        ]);
    }
}
