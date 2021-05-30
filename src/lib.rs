use std::fmt;
use std::iter::Peekable;
use std::ops;

use ncurses::*;
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

struct Item<'a> {
    icon: &'a str,
    chosen_icon: &'a str,
    chosen: bool,
    repr: String,
    preview: Option<String>,
}

impl<'a> Item<'a> {
    fn new(thing: &impl fmt::Display, icon: &'a str, chosen_icon: &'a str) -> Item<'a> {
        Item {
            icon,
            chosen_icon,
            chosen: false,
            repr: thing.to_string(),
            preview: None,
        }
    }

    fn select(&mut self) {
        self.chosen = !self.chosen;
    }

    fn chosen(&self) -> bool {
        self.chosen
    }

    fn icon(&self) -> &str {
        if self.chosen {
            self.chosen_icon
        } else {
            self.icon
        }
    }

    fn string(&self) -> &String {
        &self.repr
    }

    fn preview<D: fmt::Display>(&mut self, thing: D, func: &DispFunc<D>) {
        self.preview = Some(func.eval(thing));
    }
}

impl<'a> fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{} {}", self.icon(), self.repr))
    }
}

struct Pair {
    y: i32,
    x: i32,
}

impl Pair {
    fn clone(&self) -> Pair {
        Pair {
            y: self.y,
            x: self.x,
        }
    }
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pair({}, {})", self.y, self.x)
    }
}

#[derive(Copy, Clone)]
pub enum ScreenSide {
    Left,
    Right,
    Top,
    Bottom,
    Full,
}

impl ScreenSide {
    fn get_bounds(&self, screen_bounds: (Pair, Pair), width: f64) -> (Pair, Pair) {
        assert!(width <= 1.0 && width > 0.0);
        match self {
            Self::Top => (
                screen_bounds.0,
                Pair {
                    y: ((screen_bounds.1.y as f64) * width) as i32,
                    x: screen_bounds.1.x,
                },
            ),
            Self::Bottom => (
                // TL: height * (1 - width) + 1
                // BR: BR
                Pair {
                    y: (((screen_bounds.1.y - screen_bounds.0.y) as f64) * (1.0 - width)) as i32
                        + 1,
                    x: screen_bounds.0.x,
                },
                screen_bounds.1,
            ),
            Self::Left => (
                // TL: TL
                // BR: screen_width * width
                screen_bounds.0.clone(),
                Pair {
                    y: screen_bounds.1.y,
                    x: screen_bounds.0.x
                        + (((screen_bounds.1.x - screen_bounds.0.x) as f64) * width) as i32,
                },
            ),
            Self::Right => (
                // TL: screen_width * (1 - width) + 1
                // BR: BR
                Pair {
                    y: screen_bounds.0.y,
                    x: screen_bounds.0.x
                        + (((screen_bounds.1.x - screen_bounds.0.x) as f64) * (1.0 - width)) as i32
                        + 1,
                },
                screen_bounds.1.clone(),
            ),
            Self::Full => screen_bounds,
        }
    }
}

impl ops::Not for ScreenSide {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Full => Self::Full,
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
        self.bounds = self
            .side
            .get_bounds((Pair { y: 0, x: 0 }, Self::get_size()), self.width);
    }

    fn write_item(&mut self, item: &Item, highlight: bool) -> bool {
        self.skiplines(1);

        if self.pos.y >= self.bounds.1.y - 1 {
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

    fn draw_box(&mut self, side: ScreenSide, width: f64) {
        let bounds = side.get_bounds((self.bounds.0.clone(), self.bounds.1.clone()), width);

        let box_width = (bounds.1.x - bounds.0.x) as usize;
        // let box_height = (bounds.1.y - bounds.0.y) as usize;

        let hor_line = "─";
        let vert_line = "│";
        let corner_tl = "┌";
        let corner_bl = "└";
        let corner_tr = "┐";
        let corner_br = "┘";

        let preview_box_text = " preview ";

        // top line
        self.pos.x = bounds.0.x;
        self.pos.y = bounds.0.y;
        self.addstr(corner_tl);
        // self.addstr(hor_line);
        self.addstr(preview_box_text);
        self.addstr(&hor_line.repeat(box_width - preview_box_text.len() - 2));
        self.addstr(corner_tr);

        // vertical lines
        // accessing curses directly
        for row in bounds.0.y + 1..bounds.1.y {
            mvaddstr(row, bounds.0.x, vert_line);
            mvaddstr(row, bounds.1.x - 1, vert_line);
        }

        // bottom line
        self.pos.x = bounds.0.x;
        self.pos.y = bounds.1.y - 1;
        self.addstr(corner_bl);
        self.addstr(&hor_line.to_string().repeat(box_width - 2));
        self.addstr(corner_br);
    }

    fn get_key(&self) -> i32 {
        getch()
    }

    fn refresh(&mut self) {
        refresh();
        self.bounds = self
            .side
            .get_bounds((Pair { y: 0, x: 0 }, Self::get_size()), self.width);
    }

    fn erase(&mut self) {
        erase();
        self.bounds = self
            .side
            .get_bounds((Pair { y: 0, x: 0 }, Self::get_size()), self.width);
    }

    fn max_y(&mut self) -> usize {
        self.bounds.1.y as usize
    }

    fn _max_x(&mut self) -> usize {
        self.bounds.1.x as usize
    }

    fn reset_pos(&mut self) {
        self.pos.y = self.bounds.0.y;
        self.pos.x = self.bounds.0.x;
        self.items_on_screen = 0;
    }

    fn get_size() -> Pair {
        let mut size = Pair { y: 0, x: 0 };
        getmaxyx(stdscr(), &mut size.y, &mut size.x);
        size
    }

    fn addstr(&mut self, s: &str) {
        let screen_width = self.bounds.1.x - self.bounds.0.x;
        let mut chars = s.chars();
        let mut char_counter = 0;
        let mut curr_string = String::new();

        loop {
            let next_char = chars.next();
            if let Some(c) = next_char {
                // TODO: shorten the code here
                if char_counter >= screen_width {
                    self.addstr_clean(&curr_string);
                    curr_string.clear();
                    self.pos.y += 1;
                    self.pos.x = self.bounds.0.x;
                    char_counter = 0;
                } else if c == '\n' {
                    self.addstr_clean(&curr_string);
                    curr_string.clear();
                    self.pos.y += 1;
                    self.pos.x = self.bounds.0.x;
                    char_counter = 0;
                    continue;
                }
                if self.pos.y >= self.bounds.1.y {
                    curr_string.clear();
                    break;
                }
                curr_string.push(c);
                char_counter += 1;
            } else {
                break;
            }
        }
        self.addstr_clean(&curr_string);
    }

    // fn addstr_with_newlines(&mut self, s: &str) {
    //     for substr in s.split('\n') {
    //         mvaddstr(self.pos.y, self.pos.x, substr);
    //         self.pos.y += 1;
    //         self.pos.x = self.bounds.0.x;
    //     }
    // }

    fn addstr_clean(&mut self, s: &str) {
        mvaddstr(self.pos.y, self.pos.x, s);
        self.pos.x += s.char_indices().collect::<Vec<_>>().len() as i32;
    }

    // fn get_n(&self, chars: &mut Chars, n: usize) -> String {
    //     let mut s = String::new();
    //     for _ in 0..n {
    //         s.push_str(&chars.next().unwrap().to_string());
    //     }
    //     s
    // }

    fn addch(&mut self, c: char) {
        mvaddch(self.pos.y, self.pos.x, c as u32);
        self.pos.x += 1;
    }

    fn skiplines(&mut self, n: i32) {
        self.pos.y += n;
        self.pos.x = self.bounds.0.x;
    }

    // fn set_side(&mut self, new_side: ScreenSide) {
    //     self.side = new_side;
    // }

    // fn set_width(&mut self, new_size: f64) {
    //     self.width = new_size;
    // }
}

enum MenuReturnCode {
    Done,
    Pass,
}

struct MenuState<'a> {
    hover: usize,
    start: usize,
    items: Vec<Item<'a>>,
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

struct DispFunc<D>
where
    D: fmt::Display,
{
    func: Box<dyn Fn(D) -> String>,
}

impl<D> DispFunc<D>
where
    D: fmt::Display,
{
    fn new(func: Box<dyn Fn(D) -> String>) -> DispFunc<D> {
        DispFunc { func }
    }
    fn eval(&self, param: D) -> String {
        (*self.func)(param)
    }
}

struct Preview<D>
where
    D: fmt::Display,
{
    func: DispFunc<D>,
    box_screen: Screen,
    screen: Screen,
    side: ScreenSide,
}

impl<D> Preview<D>
where
    D: fmt::Display,
{
    fn new(func: DispFunc<D>, side: ScreenSide, width: f64) -> Preview<D> {
        let box_screen = Screen::new(side, width);
        let screen = Screen::new(side, width);

        Preview {
            func,
            side,
            box_screen,
            screen,
        }
    }

    fn draw_box(&mut self) {
        self.box_screen.draw_box(ScreenSide::Full, 1.0);
    }

    fn show(&mut self) {
        self.box_screen.show();
        self.screen.show();
        self.update_bounds();
    }

    fn update_bounds(&mut self) {
        self.screen.bounds.0.y += 1;
        self.screen.bounds.0.x += 1;
        self.screen.bounds.1.y -= 1;
        self.screen.bounds.1.x -= 1;
    }

    // fn change_pos(&mut self, side: ScreenSide, width: f64) {
    //     self.side = side;
    //     self.box_screen = Screen::new(side, width);
    //     self.screen = Screen::new(side, width);
    // }

    fn refresh(&mut self) {
        self.screen.refresh();
        self.box_screen.refresh();
        self.update_bounds();
    }
}

pub struct Menu<'a, I, D>
where
    D: fmt::Display,
    I: Iterator<Item = D>,
{
    iter: Peekable<I>,
    screen: Screen,
    preview: Option<Preview<D>>,
    item_icon: &'a str,
    chosen_item_icon: &'a str,
    selection: Vec<usize>,
    keys: Keys,

    state: MenuState<'a>,
    config: MenuConfig,
}

use MenuReturnCode::{Done, Pass};

type RetCode = MenuReturnCode;

impl<'a, I, D> Menu<'a, I, D>
where
    D: fmt::Display,
    I: Iterator<Item = D>,
{
    pub fn new(iter: I) -> Menu<'a, I, D> {
        let screen = Screen::new(ScreenSide::Full, 0.5);

        let item_icon: &'static str = ">";
        let chosen_item_icon: &'static str = "~";

        Menu {
            iter: iter.peekable(),
            screen,
            preview: None,
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
        init_curses();

        self.screen.show();
        if let Some(prev) = &mut self.preview {
            prev.show();
        }
        self.refresh();

        loop {
            match self.screen.get_key() {
                27 | 113 => break, // ESC or q

                val => {
                    // This will erase the entire window
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

        end_curses();
        self.finish()
    }

    fn finish(&self) -> Vec<usize> {
        self.selection.clone()
    }

    fn yield_item(&mut self, i: usize) -> Option<&Item> {
        while self.state.items.len() <= i {
            if let Some(item) = self.iter.next() {
                let mut new_item = Item::new(&item, self.item_icon, self.chosen_item_icon);
                if let Some(preview) = &self.preview {
                    new_item.preview(item, &preview.func);
                }
                self.state.items.push(new_item);
            } else {
                return None;
            }
        }
        Some(&self.state.items[i])
    }

    fn refresh(&mut self) {
        // Maximum index that will fit on current screen state
        let end = self.state.start + self.screen.max_y();
        self.yield_item(end);

        self.screen.reset_pos();
        if let Some(prev) = &mut self.preview {
            // prev.screen.reset_pos();
            prev.draw_box();
            prev.screen.reset_pos();
        }
        // self.preview_screen.reset_pos();
        let mut i = self.state.start;
        let pos = self.state.hover + i;
        loop {
            if let Some(item) = self.state.items.get(i) {
                if !self.screen.write_item(&item, pos == i) {
                    break;
                }
                if pos == i {
                    if let Some(prev) = &mut self.preview {
                        prev.screen.addstr(item.preview.as_ref().unwrap());
                    }
                }

                i += 1;
            } else {
                break;
            }
        }

        self.screen.refresh();

        if let Some(prev) = &mut self.preview {
            prev.refresh();
        }
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

        if new_hover < 0.0 || new_hover == num_items {
            return Pass;
        }

        self.state.hover = new_hover as usize;

        if new_hover > num_items * 0.67
            && self.state.start + self.screen.items_on_screen < self.state.items.len()
        {
            self.scroll(1);
            self.state.hover -= 1;
        } else if new_hover < num_items * 0.33 && self.state.start > 0 && amount < 0 {
            self.scroll(-1);
            self.state.hover += 1;
        }

        Pass
    }

    // CONFIG

    pub fn preview<F>(mut self, func: F) -> Menu<'a, I, D>
    where
        F: Fn(D) -> String + 'static,
    {
        let func = DispFunc::new(Box::new(func));
        self.preview = Some(Preview::new(func, ScreenSide::Right, 0.5));
        self
    }

    pub fn preview_side(mut self, side: ScreenSide) -> Menu<'a, I, D> {
        self.preview.as_mut().unwrap().side = side;
        self
    }
}

// pub fn log(s: &str) -> std::io::Result<()> {
//     let mut file = OpenOptions::new()
//         .write(true)
//         .append(true)
//         .open("choose.log")
//         .unwrap();
//     writeln!(file, "{}", s)?;
//     Ok(())
// }

pub fn init_curses() {
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

pub fn end_curses() {
    endwin();
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
