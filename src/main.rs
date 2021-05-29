mod lib;

fn main() {
    let list = "there is some text that could show up in the menu \
                           if the person that was making the menu chose to \
                               add this text it would be nice"
        .split(" ")
        .map(String::from);

    let mut menu = lib::Menu::new(list).preview_func(process);
    let choice = menu.show();
    println!("Chose {:?}", choice);
}

fn process(s: String) -> String {
    if s.len() % 2 == 0 {
        format!(
            "The word '{}' has an even number of letters. It's length is {}. You're welcome.",
            s,
            s.len()
        )
    } else {
        format!(
            "The word '{}' has an odd number of letters. It's length is {}. You're welcome.",
            s,
            s.len()
        )
    }
}
