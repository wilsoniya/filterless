#![feature(btree_range)]
#![feature(collections_bound)]
#![feature(type_ascription)]

extern crate clap;
extern crate ncurses;
extern crate libc;

pub mod buffered_filter;
pub mod iter;
pub mod pager;

use std::char;
use std::ffi::CString;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::stdin;

use clap::{Arg, App};
use libc::{fopen};
use ncurses::*;

use pager::Pager;


const LOWER_J: i32 = 0x6a;
const LOWER_K: i32 = 0x6b;
const LOWER_Q: i32 = 0x71;
const FWD_SLASH: i32 = 0x2f;
const CTRL_D: i32 = 4;
const CTRL_U: i32 = 21;
const ENTER: i32 = 10;
const BACKSPACE: i32 = 127;

const MARGIN: i32 = 0;


/// Returns a C-style string from a `&str`.
fn get_cstring(string: &str) -> CString {
    CString::new(string).unwrap()
}

/// Creates and returns an ncurses window which reads user input from tty
/// (to avoid taking commands from a piped file) and writes to stdout.
fn setup_term() -> SCREEN {
    let tty;
    let stdout;

    unsafe {
        tty = fopen(get_cstring("/dev/tty").as_ptr(),
                        get_cstring("r").as_ptr());
        stdout = fopen(get_cstring("/dev/stdout").as_ptr(),
                           get_cstring("w").as_ptr());
    }

    let term = newterm(None, stdout, tty);
    set_term(term);
    noecho();
    term
}

/// Presents a CLI and returns a boxed `std::io::BufRead` which enables
/// line-wise reading of input from a file via the CLI or failing that from
/// stdin.
///
/// ### Parameters
/// * `_stdin`: standard input from which to read if user doesn't provide a file
///   name
fn get_input<'a>(_stdin: &'a std::io::Stdin) -> Box<BufRead + 'a> {
      let matches = App::new("Filterless")
          .version(env!("CARGO_PKG_VERSION"))
          .author("Michael Wilson")
          .about("Less, but with filtering")
          .arg(Arg::with_name("INPUT")
               .help("Sets the input file to use")
               .required(false)
               .index(1))
          .get_matches();

      match matches.value_of("INPUT") {
          Some(fname) => {
              let file = File::open(fname).unwrap();
              let reader: BufReader<File> = BufReader::new(file);
              Box::new(reader)
          },
          None => Box::new(_stdin.lock())
      }
}

/// Event handler for when a user chooses to begin filtering text.
///
/// Spawns a single-line window at the bottom of the screen, collects user
/// input, and returns it after user presses ENTER.
///
/// ### Parameters
/// * `width`: width of the terminal in columns
/// * `height`: height of the terminal in rows
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
    delwin(filter_win);
    return filter_str;
}

/// System entry point
fn main() {
    let _stdin = stdin();
    let reader = get_input(&_stdin);
    let lines = reader.lines();

    let window: SCREEN = setup_term();

    let mut max_x = 0;
    let mut max_y = 0;
    getmaxyx(stdscr, &mut max_y, &mut max_x);
    let height = max_y - MARGIN;
    let width = max_x - MARGIN;

    refresh();

    let win = newwin(height, width, MARGIN / 2, MARGIN / 2);
    let mut pager = Pager::new(win);
    pager.load(lines);
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
    delscreen(window);
}

