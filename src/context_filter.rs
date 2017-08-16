use std::io::{BufRead, BufReader, Lines};
use iter::NumberedLine;

enum IterDirection {
    BACKWARD,
    FORWARD,
}

/// Thing which reads, caches, and makes filterable lines produced by linewise
/// iterators.
pub struct LineBuffer<I: Iterator<Item=String>> {
    lines: I,
    cached_lines: Vec<NumberedLine>,
    last_iter_line: usize,
    iter_direction: IterDirection,
}


impl<I: Iterator<Item=String>> LineBuffer<I> {
    /// Creates a new `LineBuffer` from a linewise iterator.
    pub fn new(iterator: I) -> LineBuffer<I> {
        LineBuffer {
            lines: iterator,
            cached_lines: Vec::new(),
            last_iter_line: 0,
            iter_direction: IterDirection::FORWARD,
        }
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

            let num_lines = line_num - last_line_num;
            let next_line_num = last_line_num + 1;
            let new_lines = (next_line_num..)
                .zip(self.lines.by_ref())
                .take(num_lines);

            self.cached_lines.extend(new_lines);

        }

        self.cached_lines.get(cache_idx).map(|i| i.to_owned())
    }

    pub fn seek(&mut self, line_num: usize, direction: IterDirection) {
        self.last_iter_line = if line_num > 1 { line_num - 1 } else { 1 };
        self.iter_direction = direction;
    }
}

impl<I: Iterator<Item=String>> Iterator for LineBuffer<I> {
    type Item = NumberedLine;

    fn next(&mut self) -> Option<Self::Item> {
        let next_line = match self.iter_direction {
            IterDirection::FORWARD => self.last_iter_line + 1,
            IterDirection::BACKWARD => {
                if self.last_iter_line > 1 { self.last_iter_line - 1 } else { 1 }
            }
        };

        self.get(next_line)
            .map(|line| {
                self.last_iter_line = next_line;
                line
            })
    }
}

mod test {
    use super::LineBuffer;
    use std;

    #[test]
    fn test() {
        let vec: Vec<String> = vec!(
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
            "four".to_owned(),
        );

//      let iter: std::slice::Iter<String> = vec.iter();
        let iter = vec.iter().cloned();
        let mut line_buf = LineBuffer::new(iter);

        let expected = Some((1, "one".to_owned()));
        let actual = line_buf.next();
        assert_eq!(expected, actual);

        let expected = Some((2, "two".to_owned()));
        let actual = line_buf.next();
        assert_eq!(expected, actual);

        let expected = Some((3, "three".to_owned()));
        let actual = line_buf.next();
        assert_eq!(expected, actual);

        let expected = Some((4, "four".to_owned()));
        let actual = line_buf.next();
        assert_eq!(expected, actual);

        let expected = None;
        let actual = line_buf.next();
        assert_eq!(expected, actual);

    }
}
