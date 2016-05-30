extern crate ncurses;

mod pager;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

use ncurses::*;

use pager::Pager;


static FNAME: &'static str = "pg730.txt";


fn main() {
    let screen: SCREEN = initscr();
    noecho();
    keypad(stdscr, true);

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr, &mut max_y, &mut max_x);
    let margin = 4i32;
    let height = max_y - margin;
    let width = max_x - margin;

    for _ in 0..(max_x * max_y) {
        printw(".");
    }

    refresh();

    let border_win = newwin(height + 2, width + 2, margin / 2 - 1, margin / 2 - 1);
    box_(border_win, 0 as u64, 0 as u64);
    wrefresh(border_win);


    let win = newwin(height, width, margin / 2, margin / 2);
    let mut pager = Pager::new(win);
    let mut file = File::open(FNAME).unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines();
    pager.load(lines);
    pager.show_line(0);

    loop {
        match getch() {
            KEY_UP => pager.prev_line(),
            KEY_DOWN => pager.next_line(),
            _ => break
        }
    }

    endwin();
    delscreen(screen);
}

fn read_file() -> Vec<String> {
    let mut file = File::open(FNAME).unwrap();
    let reader = BufReader::new(file);
    let result: Vec<String> = reader.lines().map(|s| s.unwrap()).collect();

    result
}

fn flatten_lines(lines: &[String]) -> String {
    lines.join("\n")
}
