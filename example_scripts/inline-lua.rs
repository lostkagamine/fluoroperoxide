// foof: use lua-macro latest
// foof: toolchain nightly

use lua_macro::{lua, lua_eval};

fn main() {
    lua! {
        print("hello, world lol")
    };

    let the_sum_of_2_and_2: i32 = lua_eval! { 2 + 2 };
    println!("2 + 2 is {} according to Lua", the_sum_of_2_and_2);
}