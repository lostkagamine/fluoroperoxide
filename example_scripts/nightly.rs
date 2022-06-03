// foof: toolchain nightly

#![feature(box_syntax)]

fn main() {
    let some_number = 123;
    let put_me_in_a_box = box some_number;
    println!("{}", put_me_in_a_box);
}