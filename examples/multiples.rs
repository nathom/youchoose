use youchoose;

fn main() {
    let mut menu = youchoose::Menu::new(0..100).preview(multiples).title("Multiples of integers from 0 to 99 asd;flkjas;dlfkjas;dlkfja;slkdfja;slkdjf;aslkdjf;aslkdjfa;slkdjfa;slkdjfas;lkdjfa;sldkfja;sldkjfa;slkdfja;sldkfja;lskdjfa;lskdjf;alskdjf;aslkdfj;alskdjf;alksdjf;alksdjf;alskdjf;aslkdjf;alskdfj;alskdjfa;slkdjf");
    // p
    let choice = menu.show();
    println!("Chose {:?}", choice);
}

fn multiples(num: i32) -> String {
    let mut buffer = String::new();
    for i in 0..20 {
        buffer.push_str(&format!(
            "{} times {} is equal to {}!\n",
            num,
            i,
            num * i
        ));
    }
    buffer
}
