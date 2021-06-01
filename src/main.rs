mod lib;
use std::fs;
use std::process::Command;
use std::str;

fn main() {
    // let list = "there is some text that could show up in the menu \
    //                        if the person that was making the menu chose to \
    //                            add this text it would be nice"
    //     .split(" ")
    //     .map(String::from);

    let list = get_files("./src".to_string());
    let side = lib::ScreenSide::Left;
    // let mapped = list.iter().map(|s| preview_file(s.to_string().clone()));
    let mut menu = lib::Menu::new(list.iter())
        .preview(preview_file)
        .preview_pos(side, 0.3)
        .multiselect()
        .icon(":(")
        .selected_icon(":)")
        .preview_label(" ebic ".to_string());

    let choice = menu.show();
    println!("Chose {:?}", choice);

    // println!("{:?}", mapped);
}

// fn process(s: String) -> String {
//     if s.len() % 2 == 0 {
//         format!(
//             "The word '{}' has an even number of letters. It's length is {}. You're welcome.",
//             s,
//             s.len()
//         )
//     } else {
//         format!(
//             "The word '{}' has an odd number of letters. It's length is {}. You're welcome.",
//             s,
//             s.len()
//         )
//     }
// }

fn preview_file(s: &String) -> String {
    if !s.ends_with("rs") {
        return "".to_string();
    }
    let output = Command::new("cat")
        .arg(s.clone())
        .output()
        .expect("Failed to execute command");
    let out = str::from_utf8(&output.stdout).expect(&s);
    // let err = str::from_utf8(&output.stderr).expect(&s);
    // let stdout = io::stdout();
    // let mut handle = stdout.lock();

    out.to_string()
    // handle.write_all(&*output.stdout);
    // // println!("output: {}, err: {}", out, err);
    // "".to_string()
}

fn get_files(dir: String) -> Vec<String> {
    let paths = fs::read_dir(dir).unwrap();

    let mut v = Vec::new();
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
    v
}
