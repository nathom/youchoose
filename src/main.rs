mod lib;

// #[cfg(feature = "wide")]
fn main() {
    let list: Vec<String> = vec![
        "there".to_string(),
        "are".to_string(),
        "a".to_string(),
        "few".to_string(),
        "thing".to_string(),
        "s".to_string(),
        "tha".to_string(),
        "ti".to_string(),
        "i".to_string(),
        "thought".to_string(),
        "would".to_string(),
        "be".to_string(),
        "good".to_string(),
    ];

    let mut menu = lib::Menu::new(list);
    menu.show();
}
