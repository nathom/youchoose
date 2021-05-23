use ncurses::*;
use std::cmp::min;
use std::fmt;

/*
 * Screen:
 * + write_line
 * + get_screen_size
 * + get_key
 *
 * Menu:
 * + new(Iterable)
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
        // let repr = format!("{} {}", icon, thing);
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

struct ScreenSize {
    y: i32,
    x: i32,
}

impl ScreenSize {
    fn new() -> ScreenSize {
        ScreenSize { y: -1, x: -1 }
    }

    fn update(&mut self) {
        getmaxyx(stdscr(), &mut self.y, &mut self.x);
    }
}

struct Screen {
    size: ScreenSize,
}

impl Screen {
    fn new() -> Screen {
        let mut size = ScreenSize::new();
        size.update();
        Screen { size }
    }

    fn show(&self) {
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
    }

    fn write_item(&self, row: i32, item: &Item, highlight: bool) {
        mv(row, 0);

        let icon_color = if item.chosen() { 3 } else { 2 };
        attron(COLOR_PAIR(icon_color));
        addstr(item.icon());
        attroff(COLOR_PAIR(icon_color));
        addch(' ' as u32);

        if highlight {
            attron(COLOR_PAIR(1));
        }

        addstr(item.string());

        if highlight {
            attroff(COLOR_PAIR(1));
        }
    }

    fn get_size(&self, size: &mut ScreenSize) {
        getmaxyx(stdscr(), &mut size.y, &mut size.x);
    }

    fn _wait_for_key(&self, key: i32) {
        loop {
            if getch() == key {
                break;
            }
        }
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
        self.size.update();
        self.size.y as usize
    }

    fn _max_x(&mut self) -> usize {
        self.size.update();
        self.size.x as usize
    }

    fn end(&self) {
        endwin();
    }
}

enum MenuReturnCode {
    Done,
    Pass,
}

struct MenuState {
    row: usize,
    range: (usize, usize),
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

pub struct Menu {
    list: Vec<String>,
    // list: Vec<Item<'a>>,
    screen: Screen,
    item_icon: &'static str,
    chosen_item_icon: &'static str,
    selection: Vec<usize>,
    keys: Keys,

    state: MenuState,
    config: MenuConfig,
}

use MenuReturnCode::{Done, Pass};

type RetCode = MenuReturnCode;
impl Menu {
    pub fn new(list: Vec<String>) -> Menu {
        let screen = Screen::new();

        let item_icon: &'static str = "❯";
        let chosen_item_icon: &'static str = "~";

        Menu {
            list,
            screen,
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
                row: 0,
                range: (0, 0),
                items: Vec::new(),
            },

            config: MenuConfig { multiselect: true },
        }
    }

    pub fn show(&mut self) -> Vec<usize> {
        let mut size = ScreenSize { y: -1, x: -1 };
        self.screen.show();
        self.screen.get_size(&mut size);
        self.state.range.1 = min(self.list.len(), size.y as usize);

        self.refresh();

        loop {
            match self.screen.get_key() {
                27 | 113 => break, // ESC or q

                val => {
                    self.screen.erase();
                    match self.handle_key(val) {
                        Pass => self.refresh(),
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

    fn refresh(&mut self) {
        let mut size = ScreenSize { y: -1, x: -1 };
        self.screen.get_size(&mut size);

        let slice = self.state.range.0..self.state.range.1;
        for (row, i) in slice.enumerate() {
            // TODO: compare sizes instead of retrieving value
            if let None = self.state.items.get(i) {
                self.state.items.push(Item::new(
                    &self.list[i],
                    self.item_icon,
                    self.chosen_item_icon,
                ));
            }
            let item = &self.state.items[i];
            self.screen
                .write_item(row as i32, item, self.state.row == row);
            if row > size.y as usize {
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
        let curr_item_idx = self.state.range.0 + self.state.row;
        match self.selection.last() {
            Some(&num) if num == curr_item_idx => return Done,
            _ => (),
        }
        self.state.items[curr_item_idx].select();
        self.selection.push(curr_item_idx);
        Done
    }

    fn multiselect_item(&mut self) -> RetCode {
        let curr_item_idx = self.state.range.0 + self.state.row;
        self.state.items[curr_item_idx].select();
        self.selection.push(curr_item_idx);
        Pass
    }

    // fn multiselect_item(&self) {}

    fn scroll(&mut self, amount: i32) {
        self.state.range.0 = ((self.state.range.0 as i32) + amount) as usize;
        self.state.range.1 = ((self.state.range.1 as i32) + amount) as usize;
    }

    fn move_selection(&mut self, amount: i32) -> RetCode {
        let max_y = self.screen.max_y() as f64;
        if self.state.row > (max_y * 0.67) as usize && self.state.range.1 < self.list.len() {
            self.scroll(1);
            self.state.row -= 1;
        } else if self.state.row < (max_y * 0.33) as usize && self.state.range.0 > 0 {
            self.scroll(-1);
            self.state.row += 1;
        }

        let next_row = ((self.state.row as i32) + amount) as usize;
        if next_row < min(self.list.len(), self.screen.max_y()) {
            self.state.row = next_row;
        }

        Pass
    }
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
