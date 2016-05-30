extern crate ncurses;

mod pager;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use ncurses::*;

use pager::Pager;


static FNAME: &'static str = "pg730.txt";
//static FNAME: &'static str = "Cargo.toml";
const LOWER_J: i32 = 0x6a;
const LOWER_K: i32 = 0x6b;
const LOWER_Q: i32 = 0x71;


fn main() {
    let screen: SCREEN = initscr();
    noecho();
    keypad(stdscr, true);

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr, &mut max_y, &mut max_x);
    let margin = 0i32;
    let height = max_y - margin;
    let width = max_x - margin;

    refresh();

    let file = File::open(FNAME).unwrap();
    let reader = BufReader::new(file);
    let lines = reader.lines();
    let win = newwin(height, width, margin / 2, margin / 2);

    let mut pager = Pager::new(win);
    pager.load(lines);
    pager.show_line(0);

    loop {
        match getch() {
            LOWER_J => pager.next_line(),
            LOWER_K => pager.prev_line(),
            KEY_NPAGE => pager.next_page(),
            KEY_PPAGE => pager.prev_page(),
            LOWER_Q => break,
            _ => continue

        }
    }

    endwin();
    delscreen(screen);
}
