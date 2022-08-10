use utils::deflate::*;
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    deflate_with_77(args);
}