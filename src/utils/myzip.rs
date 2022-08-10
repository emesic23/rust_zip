use super::zipfile::*;
use super::deflate::*;
use std::fs;
use std::fs::File;
use std::io::Read;

pub fn gen_zip_0(args: Vec<String>) {
    let out_name = &args[1];
    let fname = &args[2];
    let mut cur_file = File::open(fname).expect("File Not Found");
    let mut fdata: Vec<u8> = Vec::new();
    cur_file.read_to_end(&mut fdata);
    let mut zip: ZipFile;
    zip = zip_file_creator(fname.as_bytes().to_owned(), fdata, 0, None);    
    zip_writer(zip, &out_name);
}

pub fn gen_zip(args: Vec<String>) {
    let out_name = &args[1];
    let fname = &args[2];
    let mut cur_file = File::open(fname).expect("File Not Found");
    let mut fdata: Vec<u8> = Vec::new();
    cur_file.read_to_end(&mut fdata);
    let ogdata = Some(fdata.clone());
    let mut zip: ZipFile;
    let zdata = deflate_data_with_77(fdata);
    zip = zip_file_creator(fname.as_bytes().to_owned(), zdata, 8, ogdata);    
    zip_writer(zip, &out_name);
}