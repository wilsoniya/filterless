extern crate ncurses;

mod pager;

use std::fs::File;
use std::io::Read;

use ncurses::*;

use pager::Pager;


static FNAME: &'static str = "pg730.txt";
const LOWER_J: i32 = 0x6a;
const LOWER_K: i32 = 0x6b;
const LOWER_Q: i32 = 0x71;
const FWD_SLASH: i32 = 0x2f;


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

    let mut file = File::open(FNAME).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text);
    let win = newwin(height, width, margin / 2, margin / 2);

    let mut pager = Pager::new(win);
    pager.load(text);
    pager.show_line(0);

    loop {
        match getch() {
            LOWER_J => pager.next_line(),
            LOWER_K => pager.prev_line(),
            KEY_NPAGE => pager.next_page(),
            KEY_PPAGE => pager.prev_page(),
            FWD_SLASH => {
                let filter_str = _filter(width, height);
                pager.filter(filter_str);
            },
            LOWER_Q => break,
            _ => continue
        }
    }

    endwin();
    delscreen(screen);
}

fn _filter(width: i32, height: i32) -> String {
    let filter_win = newwin(1, width, height - 1, 0);
    echo();
    wprintw(filter_win, "Filter: ");
    wrefresh(filter_win);
    let mut filter_str = String::new();
    wgetstr(filter_win, &mut filter_str);
    noecho();
    delwin(filter_win);
    return filter_str;
}
