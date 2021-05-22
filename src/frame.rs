/*
 * Screen:
 * + write_line
 * + get_screen_size
 * + wait_for_key
 *
 * Menu:
 * + new(Iterable)
 * + show
 */

use ncurses::*;
use std::cmp::min;
use std::fmt;

struct FrameSize {
    y: i32,
    x: i32,
}

impl FrameSize {
    fn new() -> FrameSize {
        let mut y = 0;
        let mut x = 0;
        getmaxyx(stdscr(), &mut y, &mut x);
        FrameSize { y, x }
    }

    fn refresh(&mut self) {
        getmaxyx(stdscr(), &mut self.y, &mut self.x);
    }
}

pub struct Item {
    repr: String,
    index: i32,
    icon: String,
}

impl Item {
    pub fn new(item: impl fmt::Display, index: i32, prefix: String, icon: &String) -> Item {
        let icon = icon.clone();
        Item {
            repr: format!("{} {}{}", icon, prefix, item),
            index,
            icon,
        }
    }

    pub fn choose(&mut self, icon: String) {
        self.repr = self.repr.replace(&self.icon, &icon);
    }
}

// For testing
impl Item {
    pub fn from_string(val: &str) -> Item {
        let icon = String::from("❯");
        let prefix = String::from("pre. ");
        let index = 0;
        Item::new(val, index, prefix, &icon)
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.repr)
    }
}

pub struct Frame {
    cache: Vec<Item>,
    range: (usize, usize),
    size: FrameSize,
}

impl Frame {
    pub fn new() -> Frame {
        let locale_conf = LcCategory::all;
        setlocale(locale_conf, "en_US.UTF-8");

        initscr();
        raw();
        keypad(stdscr(), true);
        noecho();

        let size = FrameSize::new();

        Frame {
            cache: Vec::new(),
            range: (0, size.y as usize),
            size,
        }
    }

    pub fn height(&self) -> i32 {
        self.size.y
    }

    pub fn width(&self) -> i32 {
        self.size.x
    }

    pub fn refresh(&mut self) {
        self.size.refresh();

        let range = self.range.0..min(self.range.1, self.cache.len());
        for (row, i) in range.enumerate() {
            mvaddstr(row as i32, 0, &self.cache[i].to_string());
        }
    }

    pub fn send(&mut self, item: Item) {
        self.cache.push(item);
    }

    pub fn scroll(&mut self, step: Option<isize>) {
        let step = match step {
            Some(val) => val,
            None => 1,
        };
        if self.range.0 >= step && self.range.1 >= step {
            self.range.1 -= step;
            self.range.0 -= step
        }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        endwin();
    }
}
