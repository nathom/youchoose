mod lib;
use std::fs;
use std::process::Command;

fn main() {
    // let list = "there is some text that could show up in the menu \
    //                        if the person that was making the menu chose to \
    //                            add this text it would be nice"
    //     .split(" ")
    //     .map(String::from);

    let list: = Vec::new();
    get_files("./".to_string(), list);
    let side = lib::ScreenSide::Bottom;
    let mut menu = lib::Menu::new(list.iter())
        .preview(preview_file)
        .preview_side(side);
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

fn preview_file(s: &String) -> String {
    let output = Command::new("bat")
        .arg(s)
        .output()
        .expect("Failed to execute command");
    String::from_utf8(output.stdout).unwrap()
}

fn get_files(dir: String, v: &'static mut Vec<String>) {
    let paths = fs::read_dir(dir).unwrap();

    // let mut v = Vec::new();
    for path in paths {
        v.push(
            path.unwrap()
                .path()
                .as_os_str()
                .to_str()
                .unwrap()
                .to_string(),
        );
    }
}
