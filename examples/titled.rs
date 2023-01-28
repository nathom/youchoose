use youchoose;

fn main() {
    let mut menu = youchoose::Menu::new(0..100)
        .title(" Numbers from 0 to 99 ")
        .boxed();
    let choice = menu.show();
    // `choice` is a Vec<usize> containing the chosen indices
    println!("Index of the chosen item: {:?}", choice);
}
