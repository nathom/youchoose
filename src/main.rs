mod lib;

fn main() {
    let list: Vec<&str> = "there is some text that could show up in the menu \
                           if the person that was making the menu chose to \
                               add this text it would be nice"
        .split(" ")
        .collect();

    let list = list.iter().map(|s| s.to_string()).collect();
    let mut menu = lib::Menu::new(list);
    let choice = menu.show();
    println!("Chose {:?}", choice);
}
