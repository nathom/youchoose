use ncurses::*;
use std::cmp::min;
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::iter::Peekable;

/*
 * Screen:
 * + write_line
 * + get_screen_size
 * + get_key
 *
 * Menu:
 * + new(Iterator)
 * + show
 */

struct Item {
    icon: &'static str,
    chosen_icon: &'static str,
    chosen: bool,
    repr: String,
}

impl Item {
    fn new<'a>(thing: &impl fmt::Display, icon: &'static str, chosen_icon: &'static str) -> Item {
        Item {
            icon,
            chosen_icon,
            chosen: false,
            repr: thing.to_string(),
        }
    }

    fn select(&mut self) {
        self.chosen = !self.chosen;
    }

    fn chosen(&self) -> bool {
        self.chosen
    }

    fn icon(&self) -> &'static str {
        if self.chosen {
            self.chosen_icon
        } else {
            self.icon
        }
    }

    fn string(&self) -> &String {
        &self.repr
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{} {}", self.icon(), self.repr))
    }
}

struct Pair {
    y: i32,
    x: i32,
}

impl Pair {
    fn new() -> Pair {
        Pair { y: -1, x: -1 }
    }

    fn update_size(&mut self) {
        getmaxyx(stdscr(), &mut self.y, &mut self.x);
    }

    fn update_pos(&mut self) {
        getyx(stdscr(), &mut self.y, &mut self.x);
    }

    fn clone(&self) -> Pair {
        Pair {
            y: self.y,
            x: self.x,
        }
    }
}

enum ScreenSide {
    Left,
    Right,
    Top,
    Bottom,
}

impl ScreenSide {
    fn get_bounds(&self, size: Pair, width: f64) -> (Pair, Pair) {
        assert!(width < 1.0 && width > 0.0);
        match self {
            Self::Top => (
                Pair { y: 0, x: 0 },
                Pair {
                    y: ((size.y as f64) * width) as i32,
                    x: size.x,
                },
            ),
            Self::Bottom => (
                Pair {
                    y: ((size.y as f64) * (1.0 - width)) as i32,
                    x: 0,
                },
                size.clone(),
            ),
            Self::Left => (
                Pair { y: 0, x: 0 },
                Pair {
                    y: size.y,
                    x: ((size.x as f64) * width) as i32,
                },
            ),
            Self::Right => (
                Pair {
                    y: 0,
                    x: ((size.x as f64) * (1.0 - width)) as i32,
                },
                size.clone(),
            ),
        }
    }
}

struct Screen {
    bounds: (Pair, Pair),
    pos: Pair,
    items_on_screen: usize,
    side: ScreenSide,
    width: f64,
}

impl Screen {
    fn new(side: ScreenSide, width: f64) -> Screen {
        assert!(width > 0.0 && width <= 1.0);

        let bounds = (Pair { y: 0, x: 0 }, Pair { y: 0, x: 0 });
        let pos = Pair { y: 0, x: 0 };

        Screen {
            bounds,
            pos,
            items_on_screen: 0,
            side,
            width,
        }
    }

    fn show(&mut self) {
        // Allow unicode characters
        let locale_conf = LcCategory::all;
        setlocale(locale_conf, "en_US.UTF-8");
        // Create curses screen
        initscr();
        // Use default color background
        use_default_colors();
        // Do not show typed characters on screen
        noecho();
        // Allow colors
        start_color();
        // Color used to highlight hovered selection
        init_pair(1, COLOR_BLACK, COLOR_WHITE);
        // -1 means default background
        init_pair(2, COLOR_RED, -1);
        init_pair(3, COLOR_GREEN, -1);

        // Hide cursor
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

        raw();
        keypad(stdscr(), true);

        self.bounds = self.side.get_bounds(Self::get_size(), self.width);
    }

    fn write_item(&mut self, item: &Item, highlight: bool) -> bool {
        self.skiplines(1);

        if self.pos.y == self.bounds.1.y - 1 {
            return false;
        }

        let icon_color = if item.chosen() { 3 } else { 2 };

        attron(COLOR_PAIR(icon_color));
        attron(A_BOLD());

        self.addstr(item.icon());
        self.addch(' ');

        attroff(A_BOLD());
        attroff(COLOR_PAIR(icon_color));

        if highlight {
            attron(COLOR_PAIR(1));
        }

        self.addstr(item.string());

        if highlight {
            attroff(COLOR_PAIR(1));
        }

        self.items_on_screen += 1;

        true
    }

    fn get_key(&self) -> i32 {
        getch()
    }

    fn refresh(&self) {
        refresh();
    }

    fn erase(&self) {
        erase();
    }

    fn max_y(&mut self) -> usize {
        self.bounds.1.y as usize
    }

    fn _max_x(&mut self) -> usize {
        self.bounds.1.x as usize
    }

    fn curr_y(&mut self) -> usize {
        self.pos.y as usize
    }

    fn _curr_x(&mut self) -> usize {
        self.pos.x as usize
    }

    fn reset_pos(&mut self) {
        self.pos.y = self.bounds.0.y;
        self.pos.x = self.bounds.0.x;
        self.items_on_screen = 0;
    }

    fn end(&self) {
        endwin();
    }

    fn get_size() -> Pair {
        let mut size = Pair { y: 0, x: 0 };
        getmaxyx(stdscr(), &mut size.y, &mut size.x);
        size
    }

    fn addstr(&mut self, s: &str) {
        for c in s.chars() {
            mvaddch(self.pos.y, self.pos.x, c as u32);
            self.pos.x += 1;
            if self.pos.x >= self.bounds.1.x {
                self.pos.x = self.bounds.0.x;
                self.pos.y += 1;
            }
        }
    }

    fn addch(&mut self, c: char) {
        mvaddch(self.pos.y, self.pos.x, c as u32);
        self.pos.x += 1;
    }

    fn skiplines(&mut self, n: i32) {
        self.pos.y += n;
        self.pos.x = self.bounds.0.x;
    }
}

enum MenuReturnCode {
    Done,
    Pass,
}

struct MenuState {
    hover: usize,
    start: usize,
    items: Vec<Item>,
}

struct Keys {
    down: Vec<i32>,
    up: Vec<i32>,
    select: Vec<i32>,
    multiselect: Vec<i32>,
}

struct MenuConfig {
    multiselect: bool,
}

pub struct Menu<I, D>
where
    D: fmt::Display,
    I: Iterator<Item = D>,
{
    iter: Peekable<I>,
    screen: Screen,
    preview_screen: Screen,
    item_icon: &'static str,
    chosen_item_icon: &'static str,
    selection: Vec<usize>,
    keys: Keys,

    state: MenuState,
    config: MenuConfig,
}

use MenuReturnCode::{Done, Pass};

type RetCode = MenuReturnCode;

impl<I, D> Menu<I, D>
where
    D: fmt::Display,
    I: Iterator<Item = D>,
{
    pub fn new(iter: I) -> Menu<I, D> {
        let screen = Screen::new(ScreenSide::Top, 0.5);
        let preview_screen = Screen::new(ScreenSide::Right, 0.4);

        let item_icon: &'static str = ">";
        let chosen_item_icon: &'static str = "~";

        Menu {
            iter: iter.peekable(),
            screen,
            preview_screen,
            item_icon: &item_icon,
            chosen_item_icon: &chosen_item_icon,
            selection: Vec::new(),

            keys: Keys {
                down: vec![KEY_DOWN, 'j' as i32],
                up: vec![KEY_UP, 'k' as i32],
                select: vec![10],
                multiselect: vec![32],
            },

            state: MenuState {
                hover: 0,
                start: 0,
                items: Vec::new(),
            },

            config: MenuConfig { multiselect: true },
        }
    }

    pub fn show(&mut self) -> Vec<usize> {
        self.screen.show();
        self.refresh();

        loop {
            match self.screen.get_key() {
                27 | 113 => break, // ESC or q

                val => {
                    self.screen.erase();
                    match self.handle_key(val) {
                        Pass => {
                            self.refresh();
                        }
                        Done => break,
                    }
                }
            }
        }

        self.screen.end();
        self.finish()
    }

    fn finish(&self) -> Vec<usize> {
        self.selection.clone()
    }

    fn yield_item(&mut self, i: usize) -> Option<&Item> {
        while self.state.items.len() <= i {
            if let Some(item) = self.iter.next() {
                self.state
                    .items
                    .push(Item::new(&item, self.item_icon, self.chosen_item_icon));
            } else {
                return None;
            }
        }
        Some(&self.state.items[i])
    }

    fn refresh(&mut self) {
        let end = self.state.start + self.screen.max_y();
        self.yield_item(end);

        self.screen.reset_pos();
        let mut i = self.state.start;
        let pos = self.state.hover + i;
        loop {
            if let Some(item) = self.state.items.get(i) {
                if !self.screen.write_item(&item, pos == i) {
                    break;
                }
                i += 1;
            } else {
                break;
            }
        }

        self.screen.refresh();
    }

    fn handle_key(&mut self, val: i32) -> RetCode {
        if self.keys.down.contains(&val) {
            self.move_selection(1)
        } else if self.keys.up.contains(&val) {
            self.move_selection(-1)
        } else if self.config.multiselect && self.keys.multiselect.contains(&val) {
            self.multiselect_item()
        } else if self.keys.select.contains(&val) {
            self.select_item()
        } else {
            Pass
        }
    }

    fn select_item(&mut self) -> RetCode {
        let curr_item_idx = self.state.start + self.state.hover;
        match self.selection.last() {
            Some(&num) if num == curr_item_idx => return Done,
            _ => (),
        }
        self.state.items[curr_item_idx].select();
        self.selection.push(curr_item_idx);
        Done
    }

    fn multiselect_item(&mut self) -> RetCode {
        let curr_item_idx = self.state.start + self.state.hover;
        self.state.items[curr_item_idx].select();
        self.selection.push(curr_item_idx);
        Pass
    }

    fn scroll(&mut self, amount: i32) {
        self.state.start = ((self.state.start as i32) + amount) as usize;
        assert!(self.state.start < 1_000_000);
    }

    fn move_selection(&mut self, amount: i32) -> RetCode {
        let num_items = self.screen.items_on_screen as f64;
        let new_hover = ((self.state.hover as i32) + amount) as f64;
        self.state.hover = new_hover as usize;

        if new_hover > num_items * 0.67
            && self.state.start + self.screen.items_on_screen < self.state.items.len()
        {
            log("scrolling down");
            self.scroll(1);
            self.state.hover -= 1;
        } else if new_hover < num_items * 0.33 && self.state.start > 0 && amount < 0 {
            log("scrolling up");
            self.scroll(-1);
            self.state.hover += 1;
        } else {
            log("not scrolling");
        }
        Pass
    }
}

fn log(s: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("choose.log")
        .unwrap();
    writeln!(file, "{}", s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn def_item_formatting() {
        let icon = String::from("xx");
        let chosen = String::from("--");
        let mut item = Item::new("test string", &icon, &chosen);
        assert_eq!(item.to_string(), "xx test string");
    }

    #[test]
    fn chosen_item_fomatting() {
        let icon = String::from("xx");
        let chosen = String::from("--");
        let mut item = Item::new("test string", &icon, &chosen);
        item.select();
        assert_eq!(item.to_string(), "-- test string");
    }
}
