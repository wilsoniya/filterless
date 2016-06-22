use std::io::BufRead;
use std::io::Lines;

use ncurses;

use buffered_filter;

pub struct Pager<B> {
    window: ncurses::WINDOW,
    height: usize,
    width: usize,
    filter: Option<buffered_filter::BufferedFilter<B>>
}

impl<B:BufRead> Pager<B> {
    pub fn new(window: ncurses::WINDOW) -> Pager<B> {
        let mut height = 0;
        let mut width = 0;
        ncurses::getmaxyx(window, &mut height, &mut width);

        Pager {
            window: window,
            width: width as usize,
            height: height as usize,
            filter: None,
        }
    }

    pub fn load(&mut self, lines: Lines<B>) {
        self.filter = Some(buffered_filter::BufferedFilter::new(lines));
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
    }

    fn print_lines(&self, lines: Vec<buffered_filter::Line>,
                   filter_string: Option<String>) {
        ncurses::wclear(self.window);

        ncurses::start_color();
        ncurses::init_pair(1, ncurses::constants::COLOR_BLACK,
                           ncurses::constants::COLOR_YELLOW);
        ncurses::init_pair(2, ncurses::constants::COLOR_GREEN,
                           ncurses::constants::COLOR_BLACK);

        let buf_len = self.filter.as_ref().unwrap().get_buffer_length();
        let num_digits = (buf_len as f32).log10().floor() as usize + 1;

        for &(ref line_num, ref line) in lines.iter() {
            // unconditionally print line number
            ncurses::wattron(self.window, ncurses::COLOR_PAIR(2) as i32);
            ncurses::wprintw(self.window,
                             &format!("{:>1$} ", line_num + 1, num_digits));
            ncurses::wattroff(self.window, ncurses::COLOR_PAIR(2) as i32);

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
    }
}
