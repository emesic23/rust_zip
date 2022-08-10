use std::fs;
use utils::myzip::*;
use utils::myunzip::*;
use utils::inflate::*;
use std::env;
use utils::zipfile::*;
use utils::deflate::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    inflate(args);
}

#[test]
pub fn test_unzip(){
    gen_zip_0(vec!["./testdata/myzip0/zip-test2.zip".to_string(), "./testdata/myzip0/zip-test.txt".to_string()]);
    gen_unzip_0(vec!["./testdata/myzip0/zip-test2.zip".to_string()]);
}

#[test]
pub fn test_inflate() {
    inflate(vec!["".to_string(), "./testdata/inflate/fixed-huffman-overlapping-run1.deflate".to_string()]);
}

#[test]
pub fn test_inflate_dynamic() {
    inflate(vec!["".to_string(), "./testdata/inflate/dynamic-huffman-one-distance-code.deflate".to_string()]);
}

#[test]
pub fn test_deflate(){
    deflate(vec!["".to_string(), "./testdata/inflate/fixed-huffman-literals-expected".to_string()]);
    // print_bitstream(vec!["".to_string(), "./testdata/generic_data/cowsay/cowsay.txt.deflate".to_string()]);
}

#[test]
pub fn test_reinflate(){
    inflate(vec!["".to_string(), "./testdata/generic_data/cowsay/cowsay.txt.deflate".to_string()]);
}

#[test]
pub fn test_lz(){
    lz(vec!["".to_string(), "./zip-test.txt".to_string()])
}

// #[test]
// pub fn test_deflate_77(){
//     deflate_with_77(vec!["".to_string(), "./testdata/generic_data/cowsay/cowsay.txt".to_string()]);
// }

// #[test]
// pub fn test_reinflate_77(){
//     inflate(vec!["".to_string(), "./testdata/generic_data/cowsay/cowsay.txt.deflate".to_string()]);
// }