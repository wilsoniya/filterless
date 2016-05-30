use std::cmp::min;
use std::fs::File;
use std::io::BufReader;
use std::io::Lines;
use std::io::Read;

use ncurses;

pub struct Pager {
    window: ncurses::WINDOW,
    lines: Vec<String>,
    cur_line: usize,
    height: usize,
    width: usize,
}

impl Pager {
    pub fn new(window: ncurses::WINDOW) -> Pager {
        let mut height = 0;
        let mut width = 0;
        ncurses::getmaxyx(window, &mut height, &mut width);

        Pager {
            window: window,
            lines: Vec::new(),
            cur_line: 0,
            width: width as usize,
            height: height as usize,
        }
    }

    pub fn load(&mut self, lines: Lines<BufReader<File>>) {
        self.lines = lines.map(|s| s.unwrap()).collect();
    }

    pub fn show_line(&mut self, line_num: usize) {
        assert!(self.line_bounds_valid(line_num as i64));

        let start = line_num;
        let end = min(start + self.height, self.lines.len());

        let text = Self::flatten_lines(&self.lines[start..end]);
        ncurses::wclear(self.window);
        ncurses::wprintw(self.window, &text);
        ncurses::wrefresh(self.window);

        self.cur_line = line_num;
    }

    pub fn next_line(&mut self) {
        if ! self.line_bounds_valid(self.cur_line as i64 + 1) {
            return
        }
        let target_line = self.cur_line + 1;
        self.show_line(target_line);
    }

    pub fn prev_line(&mut self) {
        if ! self.line_bounds_valid(self.cur_line as i64 - 1) {
            return
        }
        let target_line = self.cur_line - 1;
        self.show_line(target_line);
    }

    fn flatten_lines(lines: &[String]) -> String {
        lines.join("\n")
    }

    fn line_bounds_valid(&self, line_num: i64) -> bool {
        (line_num >= 0) && (line_num < self.lines.len() as i64)
    }
}
