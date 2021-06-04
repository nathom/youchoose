use youchoose;

fn main(){
    let mut menu = youchoose::Menu::new(0..100).preview(multiples);
    let choice = menu.show();
    println!("Chose {:?}", choice);
    
}

fn multiples(num: i32) -> String {
    let mut buffer = String::new();
    for i in 0..20 {
        buffer.push_str(
            &format!("{} times {} is equal to {}!\n", num, i, num * i)
        );
    }
    buffer
}