use super::records::*;
use std::fs::File;
use std::io::Write;

pub struct ZipFile {
    lfrecord: LFRecord,
    cdrecord: CDRecord,
    eocdrecord: EOCDRecord,
}

pub fn zip_file_creator(fname:Vec<u8> ,fdata: Vec<u8>, comp_method: u16, ogdata: Option<Vec<u8>>) -> ZipFile {
    let fsize = fdata.len() as u32;
    let ogsize: u32;
    if comp_method == 8 && ogdata.is_some(){
        ogsize = ogdata.unwrap().len() as u32;
    }
    else{
        ogsize = fsize.clone();
    }
    let lfrecord = lfrecord_creator(comp_method, fsize, ogsize, fname.clone(), fdata);
    let cdrecord = cdrecord_creator(comp_method, fsize, ogsize, fname.clone());
    let eocdrecord = eocdrecord_creator(cdrecord_len(&cdrecord), lfrecord_len(&lfrecord));
    ZipFile { lfrecord, cdrecord, eocdrecord}
}

pub fn zip_writer(zip_file: ZipFile, write_file_name: &str) {
    let mut file = File::create(write_file_name).expect("Unable to create file");
    let mut byte_array: Vec<u8> = Vec::new();
    byte_array.extend(&lfrecord_to_byte_array(zip_file.lfrecord));
    byte_array.extend(&cdrecord_to_byte_array(zip_file.cdrecord));
    byte_array.extend(&eocdrecord_to_byte_array(zip_file.eocdrecord));
    file.write(&byte_array).expect("Unable to write zip");
}
