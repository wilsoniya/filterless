use std::io::BufRead;
use std::io::Lines;

use ncurses;

use buffered_filter::BufferedFilter;

pub struct Pager<B> {
    window: ncurses::WINDOW,
    cur_line: usize,
    height: usize,
    width: usize,
    filter: Option<BufferedFilter<B>>
}

impl<B:BufRead> Pager<B> {
    pub fn new(window: ncurses::WINDOW) -> Pager<B> {
        let mut height = 0;
        let mut width = 0;
        ncurses::getmaxyx(window, &mut height, &mut width);

        Pager {
            window: window,
            cur_line: 0,
            width: width as usize,
            height: height as usize,
            filter: None,
        }
    }

    pub fn load(&mut self, lines: Lines<B>) {
        self.filter = Some(BufferedFilter::new(lines));
    }

    pub fn next_line(&mut self) {
        self.offset_page(1);
    }

    pub fn prev_line(&mut self) {
        self.offset_page(-1);
    }

    pub fn next_page(&mut self){
        let offset = self.height as i64;
        self.offset_page(offset);
    }

    pub fn prev_page(&mut self) {
        let offset = -1 * self.height as i64;
        self.offset_page(offset);
    }

    pub fn filter(&mut self, target: String) {
        if self.filter.is_none() {
            return;
        }

        {
            let filter = self.filter.as_mut().unwrap();
            filter.set_filter(target);
        }

        self.cur_line = 0;
        self.offset_page(0);
    }

    pub fn offset_page(&mut self, line_offset: i64) {
        if self.cur_line as i64 + line_offset < 0 {
            return
        }

        match self.filter {
            Some(ref mut filter) => {
                let lines = filter.offset_to_lines(line_offset, self.height);
                let text = Self::flatten_lines(&lines);
                ncurses::wclear(self.window);
                ncurses::wprintw(self.window, &text);
                ncurses::wrefresh(self.window);
                ncurses::refresh();

                self.cur_line = (self.cur_line as i64 + line_offset) as usize;
            },
            None => (),
        }
    }

    fn flatten_lines(lines: &[String]) -> String {
        lines.join("\n")
    }
}
