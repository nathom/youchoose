use ncurses::*;
use std::cmp::min;
use std::fmt::Display;

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

struct ScreenSize {
    y: i32,
    x: i32,
}

struct Screen {}

impl Screen {
    fn new() -> Screen {
        Screen {}
    }

    fn show(&self) {
        let locale_conf = LcCategory::all;
        setlocale(locale_conf, "en_US.UTF-8");
        initscr();
        noecho();
        start_color();
        init_pair(1, COLOR_BLACK, COLOR_WHITE);
        use_default_colors();
        // raw();
        // keypad(stdscr(), true);
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    }

    fn write_line(&self, row: i32, line: &str, selected: bool) {
        if selected {
            attron(COLOR_PAIR(1));
        }
        mvaddstr(row, 0, line);
        if selected {
            attroff(COLOR_PAIR(1));
        }
    }

    fn get_size(&self, size: &mut ScreenSize) {
        getmaxyx(stdscr(), &mut size.y, &mut size.x);
    }

    fn wait_for_key(&self, key: i32) {
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
}

impl Drop for Screen {
    fn drop(&mut self) {
        endwin();
    }
}

pub struct Menu<'a> {
    list: Vec<String>,
    screen: Screen,
    item_icon: &'a str,
    chosen_item_icon: &'a str,

    down_keys: Vec<i32>,
    up_keys: Vec<i32>,
    select_keys: Vec<i32>,
    multiselect_keys: Vec<i32>,

    curr_row: usize,
    first_idx: usize,
    last_idx: usize,
}

impl Menu<'_> {
    pub fn new(list: Vec<String>) -> Menu<'static> {
        let screen = Screen::new();
        Menu {
            list,
            screen,
            item_icon: "❯",
            chosen_item_icon: "~",

            down_keys: vec![KEY_DOWN, 'j' as i32],
            up_keys: vec![KEY_UP, 'k' as i32],
            select_keys: vec![10],
            multiselect_keys: vec![20],

            curr_row: 0,
            first_idx: 0,
            last_idx: 0,
        }
    }

    pub fn show(&mut self) {
        let mut size = ScreenSize { y: -1, x: -1 };
        self.screen.show();
        self.screen.get_size(&mut size);
        self.last_idx = min(self.list.len(), size.y as usize);

        self.refresh();

        loop {
            match self.screen.get_key() {
                27 | 113 => break, // ESC or q
                val => {
                    self.handle_key(val);

                    endwin();
                    self.refresh();
                }
            }
        }
    }

    fn refresh(&mut self) {
        let mut size = ScreenSize { y: -1, x: -1 };
        self.screen.get_size(&mut size);

        for (row, item) in self.list[self.first_idx..self.last_idx].iter().enumerate() {
            let item = self.get_repr(item);
            self.screen
                .write_line(row as i32, &item, self.curr_row == row);
            if row > size.y as usize {
                break;
            }
        }

        self.screen.refresh();
    }

    fn handle_key(&mut self, val: i32) {
        if self.down_keys.contains(&val) {
            // self.scroll(-1);
            self.move_selection(1);
        } else if self.up_keys.contains(&val) {
            // self.scroll(1);
            self.move_selection(-1);
        } else if self.select_keys.contains(&val) {
            self.select_item();
        } else if self.multiselect_keys.contains(&val) {
            self.multiselect_item();
        }
        // println!(
        //     "{}, {}, {:?}, {}",
        //     self.first_idx, self.last_idx, self.list, self.curr_row
        // );
    }

    fn scroll(&mut self, amount: i32) {
        self.first_idx = ((self.first_idx as i32) + amount) as usize;
        self.last_idx = ((self.last_idx as i32) + amount) as usize;
    }

    fn move_selection(&mut self, amount: i32) {
        let temp = self.curr_row;
        self.curr_row = ((self.curr_row as i32) + amount) as usize;
        if self.curr_row >= self.list.len() {
            self.curr_row = temp;
        }
    }

    fn select_item(&mut self) {
        let selection = self.curr_row + self.first_idx;
        self.list[selection] = self.list[selection].replace(self.item_icon, self.chosen_item_icon);
    }

    fn multiselect_item(&mut self) {
        let selection = self.curr_row + self.first_idx;
        self.list[selection] = self.list[selection].replace(self.item_icon, self.chosen_item_icon);
    }

    pub fn get_repr(&self, item: impl Display) -> String {
        let mut line = String::from(self.item_icon);
        line.push_str(" ");
        line.push_str(&item.to_string());
        line
    }
}
