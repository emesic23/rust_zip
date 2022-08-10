use super::zipfile::*;
use std::fs;
use std::env;
use std::fs::File;
use std::io::Write;
use super::records::*;
use super::inflate::*;
use std::str;

pub fn gen_unzip_0(args: Vec<String>){
    let file_name: &str = &args[1];
    let mut file = File::open(file_name).expect("No File Found");
    let mut record = lfrecord_from_file(&mut file);
    if record.comp_method == 8{
        record.fname.extend(".deflate".as_bytes());
    }
    // println!("{:?}", &str::from_utf8(&record.fname.clone()).unwrap());
    let temp = &record.fname.clone();
    let split = &str::from_utf8(temp).unwrap().rsplit_once('/');
    if split.is_some(){
        fs::create_dir_all(split.unwrap().0);
    }
    println!("{:?}", &temp);
    let mut new_file = File::create(&str::from_utf8(temp).unwrap()).expect("Unable to create file");
    new_file.write_all(&record.fdata);
}

pub fn gen_unzip(args: Vec<String>) {
    let file_name: &str = &args[1];
    let mut file = File::open(file_name).expect("No File Found");
    let mut record = lfrecord_from_file(&mut file);
    let temp = &record.fname.clone();
    let split = &str::from_utf8(temp).unwrap().rsplit_once('/');
    if split.is_some(){
        fs::create_dir_all(split.unwrap().0);
    }
    let mut new_file = File::create(&str::from_utf8(temp).unwrap()).expect("Unable to create file");
    if record.comp_method == 8{
        record.fname.extend(".deflate".as_bytes());
        let inflated_data: Vec<u8> = inflate_data(record.fdata);
        new_file.write_all(&inflated_data);
    }
    else {
        new_file.write( &record.fdata);
    }
}