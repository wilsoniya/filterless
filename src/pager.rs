use std::collections::BTreeMap;
use std::io::BufRead;
use std::io::Lines;

use ncurses;

use buffered_filter;

pub struct Pager<B> {
    window: ncurses::WINDOW,
    height: usize,
    width: usize,
    filter: Option<buffered_filter::BufferedFilter<B>>,
    num_digits: usize,
    context: usize,
}

impl<B:BufRead> Pager<B> {
    pub fn new(window: ncurses::WINDOW) -> Pager<B> {
        ncurses::start_color();
        ncurses::init_pair(1, ncurses::constants::COLOR_BLACK,
                           ncurses::constants::COLOR_YELLOW);
        ncurses::init_pair(2, ncurses::constants::COLOR_GREEN,
                           ncurses::constants::COLOR_BLACK);
        ncurses::init_pair(3, ncurses::constants::COLOR_RED,
                           ncurses::constants::COLOR_BLACK);

        let mut height = 0;
        let mut width = 0;
        ncurses::getmaxyx(window, &mut height, &mut width);

        Pager {
            window: window,
            width: width as usize,
            height: height as usize,
            filter: None,
            num_digits: 1,
            context: 3,
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
        let buf_len;

        {
            let mut filter = self.filter.as_mut().unwrap();
            lines = filter.offset_to_lines(line_offset, self.height);
            filter_string = filter.get_filter();
            buf_len = filter.get_buffer_length();
        }

        self.num_digits = (buf_len as f32).log10().floor() as usize + 1;
        self.print_lines(lines, filter_string);
    }

    fn print_lines(&mut self, lines: Vec<buffered_filter::Line>,
                   filter_string: Option<String>) {
        ncurses::wclear(self.window);

        let mut line_map: BTreeMap<usize, buffered_filter::Line> = BTreeMap::new();

        // build mapping of all lines to be printed, including context lines
        {
            let filter = self.filter.as_mut().unwrap();
            let context: isize = match filter_string {
                Some(_) => self.context as isize,
                None => 0,
            };

            for &(ref line_num, ref line) in lines.iter() {
                for offset in (-1 * context)..(context + 1) {
                    let idx = *line_num as isize + offset;
                    if idx < 0 {
                        continue;
                    }
                    match filter.get_line(idx as usize) {
                        Some(line) => {
                            line_map.insert(idx as usize, line);
                        },
                        None => {
                            panic!("couldn't get line at idx {}", idx);
                        },
                    };
                }
            }
        }

        // print all lines present in mapping
        let mut last_idx = 0;
        let mut printed_lines = 0;
        for (disp_num, line) in line_map.values().enumerate() {
            if printed_lines >= self.height {
                // case: enough lines have been printed; stop printing lines
                break;
            }

            if self.context > 0 && disp_num > 0 && line.0 > last_idx + 1 {
                // case: context lines > 0  and line gap detected; show separator
                ncurses::wattron(self.window, ncurses::COLOR_PAIR(3));
                ncurses::wprintw(self.window, &format!("{:-<1$}\n", "", 79));
                ncurses::wattroff(self.window, ncurses::COLOR_PAIR(3));
                printed_lines +=1;
            }

            if printed_lines > self.height {
                // case: enough lines have been printed; stop printing lines
                break;
            }

            self.print_line(line, &filter_string);

            if disp_num < line_map.len() - 1 {
                ncurses::wprintw(self.window, "\n");
            };

            printed_lines += 1;
            last_idx = line.0
        }

        ncurses::wrefresh(self.window);
    }

    fn print_line(&self, line: &buffered_filter::Line,
                  filter_string: &Option<String>) {

        // unconditionally print line number
        ncurses::wattron(self.window, ncurses::COLOR_PAIR(2));
        ncurses::wprintw(self.window,
                         &format!("{:>1$} ", line.0 + 1, self.num_digits));
        ncurses::wattroff(self.window, ncurses::COLOR_PAIR(2));

        match filter_string {
            &Some(ref filter_string) => {
                let frags: Vec<&str> = line.1.split(filter_string).collect();

                for (i, frag) in frags.iter().enumerate() {
                    ncurses::wprintw(self.window, frag);
                    if i < frags.len() - 1 {
                        ncurses::wattron(self.window, ncurses::COLOR_PAIR(1));
                        ncurses::wprintw(self.window, filter_string);
                        ncurses::wattroff(self.window, ncurses::COLOR_PAIR(1));
                    }
                }
            },
            &None => {
                ncurses::wprintw(self.window, &line.1);
            },
        };
    }
}
