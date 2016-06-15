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
        if self.filter.is_none() {
            return;
        }

        let filter_string;
        let lines;

        {
            let mut filter = self.filter.as_mut().unwrap();
            lines = filter.offset_to_lines(line_offset, self.height);
            filter_string = filter.get_filter();
        }

        self.print_lines(lines, filter_string);
        self.cur_line = (self.cur_line as i64 + line_offset) as usize;
    }

    fn print_lines(&self, lines: Vec<String>, filter_string: Option<String>) {
        ncurses::wclear(self.window);

        ncurses::init_pair(1, ncurses::constants::COLOR_BLACK,
                           ncurses::constants::COLOR_YELLOW);
        ncurses::start_color();

        for line in lines.iter() {
            match filter_string {
                Some(ref filter_string) => {
                    let frags: Vec<&str> = line.split(filter_string).collect();

                    for (i, frag) in frags.iter().enumerate() {
                        ncurses::wprintw(self.window, frag);
                        if i < frags.len() - 1 {
                            ncurses::wattron(self.window, ncurses::COLOR_PAIR(1) as i32);
                            ncurses::wprintw(self.window, filter_string);
                            ncurses::wattroff(self.window, ncurses::COLOR_PAIR(1) as i32);
                        }
                    }
                },
                None => {
                    ncurses::wprintw(self.window, line);
                },
            }

            ncurses::wprintw(self.window, "\n");
        }

        ncurses::wrefresh(self.window);
        ncurses::refresh();
    }
}
