mod lib;

fn main() {
    let list = "there is some text that could show up in the menu \
                           if the person that was making the menu chose to \
                               add this text it would be nice"
        .split(" ")
        .map(String::from)
        .map(|s| s.repeat(20));

    let mut menu = lib::Menu::new(list);
    let choice = menu.show();
    println!("Chose {:?}", choice);
}
