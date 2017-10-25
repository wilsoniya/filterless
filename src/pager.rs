use iter::{FilteredLine, FilterPredicate, WindowBuffer};

use ncurses;

pub struct Pager<T: Iterator<Item=String>> {
    window: ncurses::WINDOW,
    height: usize,
    width: usize,
    num_digits: usize,
    window_buffer: Option<WindowBuffer<T>>,
    predicate: Option<FilterPredicate>,
}

impl<T: Iterator<Item=String>> Pager<T> {
    pub fn new(window: ncurses::WINDOW, iter: T) -> Pager<T> {
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
        ncurses::wclear(window);
        ncurses::scrollok(window, true);
        ncurses::idlok(window, true);

        let predicate = None;
        let window_buffer = WindowBuffer::new(
            iter, predicate.clone(), width as usize, height as usize);

        Pager {
            window: window,
            width: width as usize,
            height: height as usize,
            num_digits: 1,
            predicate: predicate,
            window_buffer: Some(window_buffer),
        }
    }

    pub fn next_line(&mut self) {
        let maybe_line = self.window_buffer.as_mut().and_then(|wb| {
            wb.next_line()
        });

        if let Some(filtered_line) = maybe_line {
            ncurses::wscrl(self.window, 1);
            ncurses::wmove(self.window, self.height as i32 - 1, 0);
            self.print_line(&filtered_line);
            ncurses::wrefresh(self.window);
        }
    }

    pub fn prev_line(&mut self) {
        let maybe_line = self.window_buffer.as_mut().and_then(|wb| {
            wb.prev_line()
        });

        if let Some(filtered_line) = maybe_line {
            ncurses::wscrl(self.window, -1);
            ncurses::wmove(self.window, 0, 0);
            self.print_line(&filtered_line);
            ncurses::wprintw(self.window, "\n");
            ncurses::wrefresh(self.window);
        }
    }

    pub fn next_page(&mut self){
        let maybe_lines = self.window_buffer.as_mut().map(|wb| {
            wb.next_page()
        });

        if let Some(lines) = maybe_lines {
            ncurses::wclear(self.window);

            for (i, filtered_line) in lines.iter().enumerate() {
                self.print_line(&filtered_line);

                if i < lines.len() - 1 {
                    ncurses::wprintw(self.window, "\n");
                }
            }

            ncurses::wrefresh(self.window);
        }
    }

    pub fn prev_page(&mut self) {
        let maybe_lines = self.window_buffer.as_mut().map(|wb| {
            wb.prev_page()
        });

        if let Some(lines) = maybe_lines {
            ncurses::wclear(self.window);

            for (i, filtered_line) in lines.iter().enumerate() {
                self.print_line(&filtered_line);

                if i < lines.len() - 1 {
                    ncurses::wprintw(self.window, "\n");
                }
            }

            ncurses::wrefresh(self.window);
        }
    }

    pub fn filter(&mut self, target: String) {
        let predicate = FilterPredicate {
            filter_string: target,
            context_lines: 3,
        };

        {
            let window_buffer = self.window_buffer.as_mut().expect("window_buffer is None");
            window_buffer.set_predicate(Some(predicate.clone()));
        }

        self.predicate = Some(predicate);
        self.next_page();
    }

    fn print_line_num(&mut self, line_num: usize) {
        self.num_digits = (line_num as f32).log10().floor() as usize + 1;
        ncurses::wattron(self.window, ncurses::COLOR_PAIR(2));
        ncurses::wprintw(self.window,
                         &format!("{:>1$} ", line_num, self.num_digits));
        ncurses::wattroff(self.window, ncurses::COLOR_PAIR(2));
    }

    fn print_line(&mut self, filtered_line: &FilteredLine) {
        match *filtered_line {
            FilteredLine::Gap => {
                ncurses::wprintw(self.window, "-----");
            },
            FilteredLine::ContextLine((ref line_num, ref line)) => {
                self.print_line_num(*line_num);
                ncurses::wprintw(self.window, line);

            },
            FilteredLine::MatchLine((ref line_num, ref line)) => {
                let predicate = self.predicate.as_ref().expect(
                    "Filter predicate was None.").to_owned();
                self.print_line_num(*line_num);

                let frags: Vec<&str> = line.split(&predicate.filter_string).collect();

                for (i, frag) in frags.iter().enumerate() {
                    ncurses::wprintw(self.window, frag);
                    if i < frags.len() - 1 {
                        ncurses::wattron(self.window, ncurses::COLOR_PAIR(1));
                        ncurses::wprintw(self.window, &predicate.filter_string);
                        ncurses::wattroff(self.window, ncurses::COLOR_PAIR(1));
                    }
                }
            },
            FilteredLine::UnfilteredLine((ref line_num, ref line)) => {
                self.print_line_num(*line_num);
                ncurses::wprintw(self.window, line);
            },
        }

    }
}
