use std::io::BufRead;

use iter;

struct WindowBuffer<B: BufRead> {
    buffer: iter::LineBuffer<B>,
//  iter: Box<Iterator<Item = iter::FilteredLine>>,
    predicate: Option<iter::FilterPredicate>,
    /// width of window in columns
    width: usize,
    /// height of window in lines
    height: usize,
    /// 1-offset index of line at top of window
    cur_line: usize,
}

impl<B: BufRead> WindowBuffer<B> {
    fn new(mut buffer: iter::LineBuffer<B>,
           predicate: Option<iter::FilterPredicate>,
           width: usize,
           height: usize) -> Self {

        let offset = 0;

        let mut ret = WindowBuffer {
            buffer: buffer,
//          iter: iter,
            predicate: predicate.clone(),
            width: width,
            height: height,
            cur_line: 1,
        };

//      let iter = Box::new(ret.buffer.iter(offset, predicate));

        ret
    }
}
