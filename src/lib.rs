//! # youchoose
//!
//! A simple, easy to use command line menu for Rust.
//!
//!
//!
//! ## Usage
//!
//! There are two methods you need to be familiar with to get started:
//! `Menu::new` which takes an `Iterator` as an argument, and `Menu::show`
//! which initializes `ncurses` and displays the menu.
//!
//! Here is a minimal example that displays the range  `0..100` in a menu:
//!
//! ```rust
//! let mut menu = youchoose::Menu::new(0..100);
//! let choice = menu.show();
//! // `choice` is a Vec<usize> containing the chosen indices
//! println!("Index of the chosen item: {:?}", choice);
//! ```
//!
//! ![basic config](https://raw.githubusercontent.com/nathom/youchoose/main/screenshots/basic.png)
//!
//! Either `↓↑` or `jk` can be used to scroll, and `return` is used to select.
//! `ESC` or `q` can be used to quit.
//!
//! **Previews**
//!
//! The `youchoose::Menu` has a preview feature, which executes a command and
//! shows the results on a seperate pane.
//!
//! ```rust
//! use youchoose;
//!
//! fn main(){
//!     let mut menu = youchoose::Menu::new(0..100)
//!     .preview(multiples);
//!     let choice = menu.show();
//!     println!("Chose {:?}", choice);
//!     
//! }
//!
//! fn multiples(num: i32) -> String {
//!     let mut buffer = String::new();
//!     for i in 0..20 {
//!         buffer.push_str(
//!             &format!("{} times {} is equal to {}!\n", num, i, num * i)
//!         );
//!     }
//!     buffer
//! }
//! ```
//!
//! ![preview](https://raw.githubusercontent.com/nathom/youchoose/main/screenshots/with_preview.png)
//!
//! **Customization**
//!
//! Let's take a look at an example that showcases the available methods for customization.
//!
//! ```rust
//! use youchoose;
//!
//! fn main() {
//!     let mut menu = youchoose::Menu::new(0..100)
//!         .preview(multiples)              // Sets the preview function
//!         .preview_pos(youchoose::ScreenSide::Bottom, 0.3)  // Sets the position of the preview pane
//!         .preview_label(" multiples ".to_string())    // Sets the text at the top of the preview pane
//!         .multiselect()                   // Allows multiple items to be selected
//!         .icon(":(")                      // Sets the default (not selected) icon for an item
//!         .selected_icon(":)")             // The icon for selected items
//!         .add_multiselect_key('s' as i32) // Bind the 's' key to multiselect
//!         .add_up_key('u' as i32)          // Bind the 'u' key to up
//!         .add_down_key('d' as i32)        // Bind the 'd' key to down
//!         .add_select_key('.' as i32);     // Bind the '.' key to select
//!
//!     let choice = menu.show();
//! }
//!
//! fn multiples(num: i32) -> String {
//!     // --- Snip ---
//! }
//! ```
//!
//! ![fully customized](https://raw.githubusercontent.com/nathom/youchoose/main/screenshots/customized.png)

use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::iter::Peekable;
use std::ops;

use ncurses::*;

/// A Menu that lazily displays an iterable and (optionally) its preview.
pub struct Menu<'a, I, D>
where
    D: fmt::Display,
    I: Iterator<Item = D>,
{
    title: Option<&'a str>,
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

enum MenuReturnCode {
    Done,
    Pass,
}
use MenuReturnCode::{Done, Pass};
type RetCode = MenuReturnCode;

impl<'a, I, D> Menu<'a, I, D>
where
    D: fmt::Display,
    I: Iterator<Item = D>,
{
    /// Create a new menu object. The iterable passed in must contain displayable
    /// items.
    pub fn new(iter: I) -> Self {
        let screen = Screen::new(ScreenSide::Full, 0.5);

        let item_icon: &'a str = "❯";
        let chosen_item_icon: &'a str = "*";

        Self {
            title: None,
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

            config: MenuConfig { multiselect: false },
        }
    }

    /// Initialize curses and display the menu on the screen.
    pub fn show(&mut self) -> Vec<usize> {
        init_curses();

        self.screen.show();
        if let Some(prev) = &mut self.preview {
            prev.show();
        }
        log("initial screen bounds: ");
        log(&self.screen.bounds);

        self.refresh();

        log("after refresh: ");
        log(&self.screen.bounds);
        loop {
            match self.screen.get_key() {
                27 | 113 => break, // ESC or q

                val => {
                    // This will erase the entire window
                    self.screen.erase();

                    match self.handle_key(val) {
                        Pass => {
                            self.refresh();
                            log("after refresh: ");
                            log(&self.screen.bounds);
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
                let mut new_item =
                    Item::new(&item, self.item_icon, self.chosen_item_icon);
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
        // We redraw the entire thing so that resizing the terminal will
        // properly update the menu

        if let Some(title) = self.title {
            log("has title");
            addstr(title); // Outside of both screens

            let title_height = (title.len() / self.screen.max_x() + 1) as i32;

            let top_offset = Pair {
                y: title_height,
                x: 0,
            };
            let bottom_offset = Pair { y: 0, x: 0 };

            if self.screen.bounds_offset.is_none() {
                log("settings screen offset");

                self.screen.bounds_offset =
                    Some((top_offset.clone(), bottom_offset.clone()));
            }

            if let Some(prev) = &mut self.preview {
                if prev.box_screen.bounds_offset.is_none() {
                    log("setting preview offset");
                    prev.box_screen.bounds_offset =
                        Some((top_offset, bottom_offset));

                    if let Some(offset) = prev.screen.bounds_offset.as_mut() {
                        if offset.0.y == 1 {
                            log("changing screen offset");
                            offset.0.y += title_height;
                        }
                    }
                }
                prev.screen.reset_pos();
                prev.box_screen.reset_pos();
            }
        }
        self.screen.reset_pos();

        // Maximum index that will fit on current screen state
        // because each entry will use a minimum of one line
        let end = self.state.start + self.screen.max_y();
        self.yield_item(end);

        if let Some(prev) = &mut self.preview {
            prev.draw_box();
            prev.screen.reset_pos();
        }
        let mut i = self.state.start;
        let pos = self.state.hover + i;
        while let Some(item) = self.state.items.get(i) {
            if !self.screen.write_item(&item, pos == i) {
                break;
            }
            if pos == i {
                if let Some(prev) = &mut self.preview {
                    prev.screen.addstr(item.preview.as_ref().unwrap());
                }
            }

            i += 1;
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
        } else if self.config.multiselect
            && self.keys.multiselect.contains(&val)
        {
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
        let curr_item = &mut self.state.items[curr_item_idx];
        curr_item.select();

        let item_idx_pos =
            match self.selection.iter().position(|x| *x == curr_item_idx) {
                Some(idx) => idx as i32,
                None => -1,
            };

        if curr_item.chosen() && item_idx_pos == -1 {
            self.selection.push(curr_item_idx);
        } else if !curr_item.chosen() && item_idx_pos != -1 {
            self.selection.remove(item_idx_pos as usize);
        }

        Pass
    }

    fn scroll(&mut self, amount: i32) {
        self.state.start = ((self.state.start as i32) + amount) as usize;
        assert!(self.state.start < 1_000_000);
    }

    fn move_selection(&mut self, amount: i32) -> RetCode {
        let num_items = self.screen.items_on_screen as i32;
        let new_hover = (self.state.hover as i32) + amount;

        if new_hover < 0 || num_items == new_hover {
            return Pass;
        }

        self.state.hover = new_hover as usize;
        let new_hover = new_hover as f64;
        let num_items = num_items as f64;

        if new_hover > num_items * 0.67
            && self.state.start + self.screen.items_on_screen
                < self.state.items.len()
        {
            self.scroll(1);
            self.state.hover -= 1;
        } else if new_hover < num_items * 0.33
            && self.state.start > 0
            && amount < 0
        {
            self.scroll(-1);
            self.state.hover += 1;
        }

        Pass
    }

    pub fn title(mut self, text: &'a str) -> Self {
        self.title = Some(text);
        self
    }

    /// Add a preview pane that displays the result of applying the function
    /// passed in to each item in the iterable. The function must return a
    /// String.
    pub fn preview<F>(mut self, func: F) -> Self
    where
        F: Fn(D) -> String + 'static,
    {
        let func = DispFunc::new(Box::new(func));
        self.screen.set_pos(ScreenSide::Left, 0.5);
        self.preview = Some(Preview::new(func, ScreenSide::Right, 0.5));
        self
    }

    /// Sets the position of the preview pane. The `side` parameter determines
    /// the side on which the pane sits. The `width` parameter is a float between
    /// `0.0` and `1.0`, inclusive. It determines the proportion of the screen that
    /// the preview pane should use.
    ///
    /// The menu's side is automatically switched to opposite the preview pane's side,
    /// and the menu's width is set to `1 - width`.
    pub fn preview_pos(mut self, side: ScreenSide, width: f64) -> Self {
        self.screen.set_pos(!side, 1.0 - width);
        self.preview
            .as_mut()
            .expect("Must create preview before settting it's position")
            .set_pos(side, width);

        self
    }

    /// Sets the default icon of the menu. This is displayed before each entry.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.item_icon = icon;
        self
    }

    /// Sets the icon displayed when an item is selected in multiselect mode.
    pub fn selected_icon(mut self, icon: &'a str) -> Self {
        self.chosen_item_icon = icon;
        self
    }

    /// Sets the text displayed on top of the preview box. It is recommended to surround the label
    /// with spaces for aesthetic reasons. If it is not set, `" preview "` will be used.
    pub fn preview_label(mut self, label: String) -> Self {
        self.preview
            .as_mut()
            .expect("Must create preview before settting it's position")
            .set_label(label);
        self
    }

    /// Adds a keybinding that triggers a multiselection. This inputs an `ncurses` keycode.
    /// All ascii keys can be set by passing the character as an `i32`. The keycodes for
    /// special keys can be found by importing `ncurses` and using the provided constants
    /// or by testing with the `getch` function. For example, running the following will display
    /// the keycodes on the screen.
    ///
    /// ```
    /// // use ncurses::*;
    ///
    /// initscr();
    /// loop {
    ///     let c: i32 = getch();
    ///     clear();
    ///     if c == 'q' as i32 {break}
    ///     addstr(&format!("Pressed key with keycode {}!", c.to_string()));
    /// }
    /// endwin();
    /// ```
    pub fn add_multiselect_key(mut self, key: i32) -> Self {
        self.keys.multiselect.push(key);
        self
    }

    /// Adds a keybinding that triggers an up movement. See [`add_multiselect_key`](struct.Menu.html#method.add_multiselect_key) for more information.
    pub fn add_up_key(mut self, key: i32) -> Self {
        self.keys.up.push(key);
        self
    }

    /// Adds a keybinding that triggers a down movement. See [`add_multiselect_key`](struct.Menu.html#method.add_multiselect_key) for more information.
    pub fn add_down_key(mut self, key: i32) -> Self {
        self.keys.down.push(key);
        self
    }

    /// Adds a keybinding that triggers a selection. See [`add_multiselect_key`](struct.Menu.html#method.add_multiselect_key) for more information.
    pub fn add_select_key(mut self, key: i32) -> Self {
        self.keys.select.push(key);
        self
    }

    /// Allow multiple items to be selected from the menu.
    pub fn multiselect(mut self) -> Self {
        self.config.multiselect = true;
        self
    }
}

#[derive(Debug)]
struct MenuState<'a> {
    hover: usize,
    start: usize,
    items: Vec<Item<'a>>,
}

#[derive(Debug)]
struct Keys {
    down: Vec<i32>,
    up: Vec<i32>,
    select: Vec<i32>,
    multiselect: Vec<i32>,
}

// TODO: remove this
struct MenuConfig {
    multiselect: bool,
}

#[derive(Debug)]
struct Screen {
    bounds: (Pair, Pair),
    bounds_offset: Option<(Pair, Pair)>,
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
            bounds_offset: None,
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
        self.offset_bounds();
    }

    fn offset_bounds(&mut self) {
        if let Some(offset) = &self.bounds_offset {
            self.bounds.0.y += offset.0.y;
            self.bounds.0.x += offset.0.x;
            self.bounds.1.y += offset.1.y;
            self.bounds.1.x += offset.1.x;
        }
    }

    fn write_item(&mut self, item: &Item, highlight: bool) -> bool {
        log(&self.pos);

        if self.pos.y >= self.bounds.1.y {
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
        self.skiplines(1);

        true
    }

    fn draw_box(
        &mut self,
        side: ScreenSide,
        width: f64,
        label: &Option<String>,
    ) {
        let bounds = side
            .get_bounds((self.bounds.0.clone(), self.bounds.1.clone()), width);

        let box_width = (bounds.1.x - bounds.0.x) as usize;
        // let box_height = (bounds.1.y - bounds.0.y) as usize;

        let hor_line = "─";
        let vert_line = "│";
        let corner_tl = "┌";
        let corner_bl = "└";
        let corner_tr = "┐";
        let corner_br = "┘";

        // top line
        self.pos.x = bounds.0.x;
        self.pos.y = bounds.0.y;
        self.addstr(corner_tl);
        let label_len = match label {
            Some(label) => {
                self.addstr(&label);
                label.len()
            }
            None => {
                self.addstr(" preview ");
                9
            }
        };
        self.addstr(&hor_line.repeat(box_width - label_len - 2));
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
        self.offset_bounds();
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

    fn max_x(&mut self) -> usize {
        self.bounds.1.x as usize
    }

    fn reset_pos(&mut self) {
        self.offset_bounds();
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
        let chars = s.chars();
        let mut char_counter = 0;
        let mut curr_string = String::new();

        for c in chars {
            // TODO: shorten the code here
            let mut both = false;
            if char_counter >= screen_width {
                self.addstr_clean(&curr_string);
                curr_string.clear();
                self.pos.y += 1;
                self.pos.x = self.bounds.0.x;
                char_counter = 0;

                both = true;
            }
            if c == '\n' {
                self.addstr_clean(&curr_string);
                curr_string.clear();
                self.pos.y += 1;
                self.pos.x = self.bounds.0.x;
                char_counter = 0;
                both &= true;
                if both {
                    self.pos.y -= 1;
                }

                continue;
            }
            if self.pos.y >= self.bounds.1.y {
                curr_string.clear();
                break;
            }
            curr_string.push(c);
            char_counter += 1;
        }

        assert!(!curr_string.contains('\n'));
        self.addstr_clean(&curr_string);
    }

    fn addstr_clean(&mut self, s: &str) {
        mvaddstr(self.pos.y, self.pos.x, s);
        self.pos.x += s.char_indices().count() as i32;
    }

    fn addch(&mut self, c: char) {
        mvaddch(self.pos.y, self.pos.x, c as u32);
        self.pos.x += 1;
    }

    fn skiplines(&mut self, n: i32) {
        self.pos.y += n;
        self.pos.x = self.bounds.0.x;
    }

    fn set_pos(&mut self, new_side: ScreenSide, width: f64) {
        assert!(width > 0.0 && width <= 1.0);

        self.side = new_side;
        self.width = width;
    }
}

#[derive(Debug)]
struct Item<'a> {
    icon: &'a str,
    chosen_icon: &'a str,
    chosen: bool,
    repr: String,
    preview: Option<String>,
}

impl<'a> Item<'a> {
    fn new(
        thing: &impl fmt::Display,
        icon: &'a str,
        chosen_icon: &'a str,
    ) -> Item<'a> {
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

#[derive(Debug, Clone)]
struct Pair {
    y: i32,
    x: i32,
}

impl Pair {
    fn zeroed() -> Self {
        Self { y: 0, x: 0 }
    }
}

impl ops::Add for Pair {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            y: self.y + other.y,
            x: self.x + other.x,
        }
    }
}
impl ops::Sub for Pair {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            y: self.y - other.y,
            x: self.x - other.x,
        }
    }
}

#[derive(Debug, Clone)]
struct Bounds(Pair, Pair);

impl Bounds {
    fn zeroed() -> Self {
        Self(Pair::zeroed(), Pair::zeroed())
    }
}

/// Determines the side on which a pane should be located.
#[derive(Debug, Copy, Clone)]
pub enum ScreenSide {
    Left,
    Right,
    Top,
    Bottom,
    /// This option is not affected by width. It will always fill the screen.
    Full,
}

impl ScreenSide {
    fn get_bounds(
        &self,
        screen_bounds: (Pair, Pair),
        width: f64,
    ) -> (Pair, Pair) {
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
                    y: (((screen_bounds.1.y - screen_bounds.0.y) as f64)
                        * (1.0 - width)) as i32
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
                        + (((screen_bounds.1.x - screen_bounds.0.x) as f64)
                            * width) as i32,
                },
            ),
            Self::Right => (
                // TL: screen_width * (1 - width) + 1
                // BR: BR
                Pair {
                    y: screen_bounds.0.y,
                    x: screen_bounds.0.x
                        + (((screen_bounds.1.x - screen_bounds.0.x) as f64)
                            * (1.0 - width)) as i32
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
    label: Option<String>,
}

impl<D> Preview<D>
where
    D: fmt::Display,
{
    fn new(func: DispFunc<D>, side: ScreenSide, width: f64) -> Self {
        let box_screen = Screen::new(side, width);

        let mut screen = Screen::new(side, width);
        screen.bounds_offset =
            Some((Pair { y: 1, x: 1 }, Pair { y: -1, x: -1 }));

        Self {
            func,
            box_screen,
            screen,
            label: None,
        }
    }

    fn draw_box(&mut self) {
        log("drawing box with bounds");
        log(&self.box_screen);
        log("normal screen bouds");
        log(&self.screen);
        self.box_screen.draw_box(ScreenSide::Full, 1.0, &self.label);
    }

    fn show(&mut self) {
        self.box_screen.show();
        self.screen.show();
    }

    //     fn update_bounds(&mut self) {
    //         self.screen.bounds.0.y += 1;
    //         self.screen.bounds.0.x += 1;
    //         self.screen.bounds.1.y -= 1;
    //         self.screen.bounds.1.x -= 1;
    //     }

    fn refresh(&mut self) {
        self.screen.refresh();
        self.box_screen.refresh();
    }

    fn set_pos(&mut self, side: ScreenSide, width: f64) {
        self.screen.set_pos(side, width);
        self.box_screen.set_pos(side, width);
    }

    fn set_label(&mut self, label: String) {
        self.label = Some(label);
    }
}
fn log(s: impl fmt::Debug) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("choose.log")
        .unwrap();
    writeln!(file, "{:?}", s).unwrap();
}

fn init_curses() {
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

fn end_curses() {
    endwin();
}
