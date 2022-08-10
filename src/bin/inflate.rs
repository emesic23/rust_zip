use std::env;
use utils::inflate::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    inflate(args);
}