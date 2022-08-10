use utils::myzip::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    gen_zip_0(args);
}