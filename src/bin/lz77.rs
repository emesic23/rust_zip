use std::env;
use utils::deflate::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    lz(args);
}
