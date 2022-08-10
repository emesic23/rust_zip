use utils::myunzip::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    gen_unzip_0(args);
}