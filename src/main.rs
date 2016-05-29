extern crate ncurses;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

use ncurses::*;


static fname: &'static str = "Cargo.toml";


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
    wprintw(win, &flatten_lines(&read_file()));
    wrefresh(win);

    getch();
    wclear(win);
    wrefresh(win);

    getch();
    endwin();

    delscreen(screen);

    writeln!(&mut std::io::stderr(), "max_x: {}, max_y: {}", max_x, max_y);
    writeln!(&mut std::io::stderr(), "width: {}, height: {}", height, width);
}

fn read_file() -> Vec<String> {
    let mut file = File::open(fname).unwrap();
    let reader = BufReader::new(file);
    let result: Vec<String> = reader.lines().map(|s| s.unwrap()).collect();

    result
}

fn flatten_lines(lines: &[String]) -> String {
    lines.join("\n")
}
