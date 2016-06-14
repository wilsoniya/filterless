#![feature(btree_range)]
#![feature(collections_bound)]
#![feature(type_ascription)]

extern crate clap;
extern crate ncurses;

mod buffered_filter;
mod pager;

use std::char;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use ncurses::*;
use clap::{Arg, App, SubCommand};

use pager::Pager;


const LOWER_J: i32 = 0x6a;
const LOWER_K: i32 = 0x6b;
const LOWER_Q: i32 = 0x71;
const FWD_SLASH: i32 = 0x2f;
const CTRL_D: i32 = 4;
const CTRL_U: i32 = 21;
const ENTER: i32 = 10;
const BACKSPACE: i32 = 127;


fn main() {

	let matches = App::new("Filterless")
		.version("1.0")
		.author("Michael Wilson")
		.about("Less, but with filtering")
		.arg(Arg::with_name("INPUT")
			 .help("Sets the input file to use")
			 .required(true)
			 .index(1))
        .get_matches();

    let fname = matches.value_of("INPUT").unwrap();
    let file = File::open(fname).unwrap();
    let reader = BufReader::new(file);

    let screen: SCREEN = initscr();
    noecho();
//  keypad(stdscr, true);

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr, &mut max_y, &mut max_x);
    let margin = 0i32;
    let height = max_y - margin;
    let width = max_x - margin;

    refresh();

    let win = newwin(height, width, margin / 2, margin / 2);

    let mut pager = Pager::new(win);
    pager.load(reader.lines());
    pager.offset_page(0);

    loop {
        match getch() {
            LOWER_J => pager.next_line(),
            LOWER_K => pager.prev_line(),
            KEY_NPAGE | CTRL_D => pager.next_page(),
            KEY_PPAGE | CTRL_U => pager.prev_page(),
            FWD_SLASH => {
                let filter_str = _filter(width, height);
                pager.filter(filter_str);
            },
            LOWER_Q => break,
            _ => continue,
        }
    }

    endwin();
    delscreen(screen);
}

fn _filter(width: i32, height: i32) -> String {
    let filter_win = newwin(1, width, height - 1, 0);
    wprintw(filter_win, "Filter: ");
    wrefresh(filter_win);
    let mut filter_str = String::new();
    loop {
        match getch() {
            ENTER => break,
            BACKSPACE => {
                match filter_str.pop() {
                    Some(_) => {
                        let mut x = 0;
                        let mut y = 0;
                        getyx(filter_win, &mut y, &mut x);
                        wmove(filter_win, y, x - 1);
                        wdelch(filter_win);
                        wrefresh(filter_win);
                    },
                    None => {},
                }
            },
            ch => {
                filter_str.push(char::from_u32(ch as u32).unwrap());
                waddch(filter_win, ch as chtype);
                wrefresh(filter_win);
            },
        }
    }
//  wgetstr(filter_win, &mut filter_str);
//  noecho();
    delwin(filter_win);
    return filter_str;
}
