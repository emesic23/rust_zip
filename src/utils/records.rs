use std::fs::File;
use std::io::Write;
use std::fs;
use std::io::Read;
use std::io::{self, prelude::*, SeekFrom};
use std::convert::TryInto;
pub struct LFRecord {
    pub lf_sig: u32,
    pub e_ver: u16,
    pub gen_flag: u16,
    pub comp_method: u16,
    pub last_mod_time: u16,
    pub last_mod_date: u16,
    pub crc_32: u32,
    pub comp_fsize: u32,
    pub uncomp_fsize: u32,
    pub fname_len: u16,
    pub exfield_len: u16,
    pub fname: Vec<u8>,
    pub exfield: Option<Vec<u8>>,
    pub fdata: Vec<u8>
} 

pub struct CDRecord {
    cd_sig: u32,
    spec_ver: u8,
    made_by: u8,
    extract_ver: u16,
    gen_flag: u16,
    pub comp_method: u16,
    last_mod_time: u16,
    last_mod_date: u16,
    crc_32: u32,
    comp_fsize: u32,
    ncomp_fsize: u32,
    fname_len: u16,
    exfield_len: u16,
    f_comment_len: u16,
    disk_num_start: u16,
    int_file_attr: u16,
    ext_file_attr: u32,
    offset_local_head: u32,
    pub fname: Vec<u8>,
    extra_field: Option<Vec<u8>>,
    file_comment: Option<Vec<u8>>
}

pub struct EOCDRecord {
    eocd_signature: u32,
    disk_num: u16,
    start_disk_num: u16,
    tot_entries_on_disk: u16,
    tot_entries: u16,
    cdr_size: u32,
    cdr_offset: u32,
    file_comment_len: u16,
    file_comment: Option<Vec<u8>>
}

pub fn lfrecord_creator(comp_method: u16, comp_fsize: u32, uncomp_fsize:u32, fname: Vec<u8>, fdata: Vec<u8>) -> LFRecord {
    LFRecord {
        lf_sig: 0x04034b50,
        e_ver: 20,
        gen_flag: 0,
        comp_method: comp_method,
        last_mod_time: 0,
        last_mod_date: 0,
        crc_32: 0xdeadbeef,
        comp_fsize: comp_fsize,
        uncomp_fsize: uncomp_fsize,
        fname_len: fname.len() as u16,
        exfield_len: 0,
        fname: fname,
        exfield: None,
        fdata
    }
}

pub fn cdrecord_creator(comp_method: u16, comp_fsize: u32, ncomp_fsize: u32, fname: Vec<u8>) -> CDRecord {
    CDRecord{
        cd_sig: 0x02014b50,
        spec_ver: 30,
        made_by: 65,
        extract_ver: 20,
        gen_flag: 0,
        comp_method: comp_method,
        last_mod_time: 0,
        last_mod_date: 0,
        crc_32: 0xdeadbeef,
        comp_fsize: comp_fsize,
        ncomp_fsize: ncomp_fsize,
        fname_len: fname.len() as u16,
        exfield_len: 0,
        f_comment_len: 0,
        disk_num_start: 0,
        int_file_attr: 1,
        ext_file_attr: 1,
        offset_local_head: 0,
        fname: fname,
        extra_field: None,
        file_comment: None
    }
}

pub fn eocdrecord_creator(cdr_size: u32, cdr_offset: u32) -> EOCDRecord{
    EOCDRecord{
        eocd_signature: 0x06054b50,
        disk_num: 0,
        start_disk_num: 0,
        tot_entries_on_disk: 1,
        tot_entries: 1,
        cdr_size: cdr_size,
        cdr_offset: cdr_offset,
        file_comment_len: 0,
        file_comment: None
    }
}

const  CDRECORD_BASE_SIZE: u32 = 46;
const  LFRECORD_BASE_SIZE: u32 = 30;

pub fn cdrecord_len(cdrecord: &CDRecord) -> u32 {
    CDRECORD_BASE_SIZE + (cdrecord.fname.len() as u32)
}

pub fn lfrecord_len(lfrecord: &LFRecord) -> u32 {
    LFRECORD_BASE_SIZE + (lfrecord.fname.len() as u32) + (lfrecord.fdata.len() as u32)
}

pub fn lfrecord_to_byte_array(lfrecord: LFRecord) -> Vec<u8> {
    let mut array: Vec<u8> = Vec::new();
    array.extend_from_slice(&lfrecord.lf_sig.to_le_bytes());
    array.extend_from_slice(&lfrecord.e_ver.to_le_bytes());
    array.extend_from_slice(&lfrecord.gen_flag.to_le_bytes());
    array.extend_from_slice(&lfrecord.comp_method.to_le_bytes());
    array.extend_from_slice(&lfrecord.last_mod_time.to_le_bytes());
    array.extend_from_slice(&lfrecord.last_mod_date.to_le_bytes());
    array.extend_from_slice(&lfrecord.crc_32.to_le_bytes());
    array.extend_from_slice(&lfrecord.comp_fsize.to_le_bytes());
    array.extend_from_slice(&lfrecord.uncomp_fsize.to_le_bytes());
    array.extend_from_slice(&lfrecord.fname_len.to_le_bytes());
    array.extend_from_slice(&lfrecord.exfield_len.to_le_bytes());
    array.extend_from_slice(&lfrecord.fname);
    if lfrecord.exfield_len != 0 {
        array.extend_from_slice(&lfrecord.exfield.unwrap());
    }
    array.extend_from_slice(&lfrecord.fdata);
    return array;
}

pub fn cdrecord_to_byte_array(cdrecord: CDRecord) -> Vec<u8> {
    let mut array: Vec<u8> = Vec::new();
    array.extend_from_slice(&cdrecord.cd_sig.to_le_bytes());
    array.extend_from_slice(&cdrecord.spec_ver.to_le_bytes());
    array.extend_from_slice(&cdrecord.made_by.to_le_bytes());
    array.extend_from_slice(&cdrecord.extract_ver.to_le_bytes());
    array.extend_from_slice(&cdrecord.gen_flag.to_le_bytes());
    array.extend_from_slice(&cdrecord.comp_method.to_le_bytes());
    array.extend_from_slice(&cdrecord.last_mod_time.to_le_bytes());
    array.extend_from_slice(&cdrecord.last_mod_date.to_le_bytes());
    array.extend_from_slice(&cdrecord.crc_32.to_le_bytes());
    array.extend_from_slice(&cdrecord.comp_fsize.to_le_bytes());
    array.extend_from_slice(&cdrecord.ncomp_fsize.to_le_bytes());
    array.extend_from_slice(&cdrecord.fname_len.to_le_bytes());
    array.extend_from_slice(&cdrecord.exfield_len.to_le_bytes());
    array.extend_from_slice(&cdrecord.f_comment_len.to_le_bytes());
    array.extend_from_slice(&cdrecord.disk_num_start.to_le_bytes());
    array.extend_from_slice(&cdrecord.int_file_attr.to_le_bytes());
    array.extend_from_slice(&cdrecord.ext_file_attr.to_le_bytes());
    array.extend_from_slice(&cdrecord.offset_local_head.to_le_bytes());
    array.extend_from_slice(&cdrecord.fname);
    if cdrecord.exfield_len != 0 {
        array.extend_from_slice(&cdrecord.extra_field.unwrap());
    }
    if cdrecord.f_comment_len != 0 {
        array.extend_from_slice(&cdrecord.file_comment.unwrap());
    }
    return array;
}

pub fn eocdrecord_to_byte_array(eocdrecord: EOCDRecord) -> Vec<u8> {
    let mut array: Vec<u8> = Vec::new();
    array.extend_from_slice(&eocdrecord.eocd_signature.to_le_bytes());
    array.extend_from_slice(&eocdrecord.disk_num.to_le_bytes());
    array.extend_from_slice(&eocdrecord.start_disk_num.to_le_bytes());
    array.extend_from_slice(&eocdrecord.tot_entries_on_disk.to_le_bytes());
    array.extend_from_slice(&eocdrecord.tot_entries.to_le_bytes());
    array.extend_from_slice(&eocdrecord.cdr_size.to_le_bytes());
    array.extend_from_slice(&eocdrecord.cdr_offset.to_le_bytes());
    array.extend_from_slice(&eocdrecord.file_comment_len.to_le_bytes());

    if eocdrecord.file_comment_len != 0 {
        array.extend_from_slice(&eocdrecord.file_comment.unwrap());
    }
    return array;
}

pub fn read_2bytes(file: &mut File, buffer: &mut [u8]) -> u16{
    file.read(buffer).expect("Unable to read?");
    let num = u16::from_le_bytes(buffer.try_into().expect("Wrong slice size"));
    return num
}

pub fn read_4bytes(file: &mut File, buffer: &mut [u8]) -> u32{
    file.read(buffer).expect("Unable to read?");
    let num = u32::from_le_bytes(buffer.try_into().expect("Wrong slice size"));
    return num
}

pub fn read_string(file: &mut File, buffer: &mut Vec<u8>) -> String{
    file.read(buffer).expect("Unable to read?");
    let string = String::from_utf8(buffer.clone()).expect("INVALID READ");
    return string;
}

pub fn lfrecord_from_file(file: &mut File) -> LFRecord{
    let mut buf4 = [0; 4];
    let mut buf2 = [0; 2];
    let lf_sig = read_4bytes(file, &mut buf4);
    let e_ver = read_2bytes(file, &mut buf2);
    let gen_flag = read_2bytes(file, &mut buf2);
    let comp_method = read_2bytes(file, &mut buf2);
    let last_mod_file_time = read_2bytes(file, &mut buf2);
    let last_mod_file_date = read_2bytes(file, &mut buf2);
    let crc32 = read_4bytes(file, &mut buf4);
    let comp_file_size = read_4bytes(file, &mut buf4);
    let uncomp_file_size = read_4bytes(file, &mut buf4);
    let file_name_length = read_2bytes(file, &mut buf2);
    let extra_field_length = read_2bytes(file, &mut buf2);
    let mut fname_buff: Vec<u8> = vec![0; file_name_length as usize];
    file.read(&mut fname_buff).expect("Unable to read?");
    let mut extra_buff = vec![0; extra_field_length as usize];
    file.read(&mut extra_buff).expect("Unable to read?");
    let mut fdata_buff: Vec<u8>;
    if comp_method == 0{
        fdata_buff = vec![0; uncomp_file_size as usize];
    }
    else{
        fdata_buff = vec![0; comp_file_size as usize];
    }
    file.read(&mut fdata_buff).expect("unable to read");
    let record = LFRecord {
        lf_sig: lf_sig,
        e_ver: e_ver,
        gen_flag: gen_flag,
        comp_method: comp_method,
        last_mod_time: last_mod_file_time,
        last_mod_date: last_mod_file_date,
        crc_32: crc32,
        comp_fsize: comp_file_size,
        uncomp_fsize: uncomp_file_size,
        fname_len: file_name_length,
        exfield_len: extra_field_length,
        fname: fname_buff.clone(),
        exfield: Some(extra_buff.clone()),
        fdata: fdata_buff.clone()
    };
    return record;
}

