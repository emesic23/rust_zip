use std::fs::File;
use std::io::Read;
use std::io::Write;
use super::helpers::*;
use std::collections::HashMap;

struct BitStreamDeflator{
    data: Vec<u8>,
    lz77map: HashMap<Vec<u8>, Vec<usize>>,
    data_pos: usize,
    lz_pos: usize,
    cur_block_type: usize,
    look_back_buffer: Vec<u8>,
    bit_buffer: Vec<bool>,
    finished: bool,
    lz77_output: Vec<usize>,
    lz77_internal: Vec<(usize, usize, usize)>,
}


impl BitStreamDeflator {

    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            lz77map: HashMap::new(),
            data_pos: 0,
            lz_pos: 0,
            cur_block_type: 0,
            look_back_buffer: Vec::new(),
            bit_buffer: Vec::new(),
            finished: false,
            lz77_output: Vec::new(),
            lz77_internal: Vec::new(),
            
        }
    }

    fn read_next(&mut self) -> usize {
        let ret = self.data[self.data_pos] as usize;
        self.data_pos += 1;
        return ret
    }

    fn read_triplet(&mut self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        while(buffer.len() != 3){
            buffer.push(self.data[self.data_pos + buffer.len()]);
        }
        return buffer;
    }

    fn read_triplet_at(&self, at: usize) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        while(buffer.len() != 3){
            buffer.push(self.data[at + buffer.len()]);
        }
        return buffer;
    }

    fn read_next_from(&self, loc: usize) -> u8 {
        return self.data[loc];
    }

    fn lz77_read_next(&mut self) -> Option<(usize, usize, usize)>{
        if self.lz_pos < self.lz77_internal.len() {
            let ret = self.lz77_internal[self.lz_pos];
            self.lz_pos += 1;
            return Some(ret);
        }
        else {
            return None
        }
    }

    const EOB_CODE: usize = 256;

    const MATCH_7_LOW: usize = 0;
    const MATCH_7_HIGH: usize = 23;
    const MATCH_8_1_LOW: usize = 48;
    const MATCH_8_1_HIGH: usize = 191;
    const MATCH_8_2_LOW: usize = 192;
    const MATCH_8_2_HIGH: usize = 199;
    const MATCH_9_LOW: usize = 400;
    const MATCH_9_HIGH: usize = 511;

    const BASE_7_CODE: usize = 256;
    const BASE_8_1_CODE: usize = 0;
    const BASE_8_2_CODE: usize = 280;
    const BASE_9_CODE: usize = 144;

    const BFINAL: bool = true;
    const BTYPE: [bool; 2]= [true, false];

    fn write_huffman(&mut self){
        self.bit_buffer.push(BitStreamDeflator::BFINAL);
        self.bit_buffer.extend(Vec::from(BitStreamDeflator::BTYPE));
        while self.data_pos < self.data.len() {
            let cur_val = self.read_next();
            
            let mut cur_code = cur_val + BitStreamDeflator::MATCH_8_1_LOW - BitStreamDeflator::BASE_8_1_CODE;
            let size: u8;
            if BitStreamDeflator::MATCH_8_1_LOW <= cur_code && cur_code <= BitStreamDeflator::MATCH_8_1_HIGH {
                size = 8;
            }
            else{
                cur_code = cur_val + BitStreamDeflator::MATCH_9_LOW - BitStreamDeflator::BASE_9_CODE;
                size = 9;
            }
            let bits = usize_to_bits(cur_code, size);
            self.bit_buffer.extend(bits);
        }
        self.bit_buffer.extend(vec![false; 7]);  

        self.prep_look_back_buffer();
    }

    fn prep_look_back_buffer(&mut self) {
        let mut byte_buffer: Vec<bool> = Vec::new();

        for (i, b) in self.bit_buffer.iter().enumerate() {
            if (i % 8) == 0 && i > 0 {
                byte_buffer.reverse();
                self.look_back_buffer.push(binary_to_dec(&byte_buffer) as u8);
                byte_buffer = Vec::new()
            }
            byte_buffer.push(*b);
        }
        
        // byte_buffer.reverse();
        while (byte_buffer.len() != 8 && byte_buffer.len() > 0) {
            byte_buffer.push(false);
        }
        self.look_back_buffer.push(binary_to_dec(&byte_buffer) as u8);
    }

    fn write_huffman_with_lz77(&mut self) {
        self.bit_buffer.push(BitStreamDeflator::BFINAL);
        self.bit_buffer.extend(Vec::from(BitStreamDeflator::BTYPE));
        let mut next_skip = self.lz77_read_next();

        while self.data_pos < self.data.len() {
            let cur_val = self.read_next();

            match next_skip {
                Some((pos, amount_skip, dist_back)) if pos == self.data_pos - 1 => {
                    self.data_pos += amount_skip - 1;
                    next_skip = self.lz77_read_next();

                    let mut extra_bit_len: usize = 0;
                    let mut extra_bit_val: usize = 0;
                    let mut cur_val: usize = 0;
                    
                    

                    match amount_skip {
                        3..=10 => {cur_val = amount_skip + 254; extra_bit_len = 0; extra_bit_val = 0;}
                        11..=18 => {cur_val = ((amount_skip - 11) / 2) + 265; extra_bit_len = 1; extra_bit_val = (amount_skip + 1) % 2;}
                        19..=34 => {cur_val = ((amount_skip - 19) / 4) + 269; extra_bit_len = 2; extra_bit_val = (amount_skip + 1) % 4;}
                        35..=66 => {cur_val = ((amount_skip - 35) / 8) + 273; extra_bit_len = 3; extra_bit_val = (amount_skip + 5) % 8;}
                        67..=130 => {cur_val = ((amount_skip - 67) / 16) + 277; extra_bit_len = 4; extra_bit_val = (amount_skip + 13) % 16;}
                        131..=257 => {cur_val = ((amount_skip - 131) / 32) + 281; extra_bit_len = 5; extra_bit_val = (amount_skip + 29) % 32;}
                        258 => {cur_val = 285; extra_bit_len = 0; extra_bit_val = 0;}
                        _ => {}
                    }
                    let size: u8;
                    let mut cur_code = cur_val + BitStreamDeflator::MATCH_7_LOW - BitStreamDeflator::BASE_7_CODE;
                    if BitStreamDeflator::MATCH_7_LOW <= cur_code && cur_code <= BitStreamDeflator::MATCH_7_HIGH {
                        size = 7;
                    }
                    else{
                        cur_code = cur_val + BitStreamDeflator::MATCH_8_2_LOW - BitStreamDeflator::BASE_8_2_CODE;
                        size = 8;
                    }
                    let bits = usize_to_bits(cur_code, size);
                    self.bit_buffer.extend(bits);
                    if extra_bit_len > 0 {
                        let mut extra_bits = usize_to_bits(extra_bit_val, extra_bit_len as u8);
                        extra_bits.reverse();
                        self.bit_buffer.extend(extra_bits);
                    }

                    let mut extra_bit_len: usize = 0;
                    let mut extra_bit_val: usize = 0;
                    let mut cur_val: usize = 0;

                    match dist_back{
                        1..=4 => {cur_val = dist_back - 1; extra_bit_len = 0; extra_bit_val = 0;}
                        5..=8 => {cur_val = ((dist_back - 5) / 2) + 4; extra_bit_len = 1; extra_bit_val = (dist_back + 1) % 2;}
                        9..=16 => {cur_val = ((dist_back - 9) / 4) + 6; extra_bit_len = 2; extra_bit_val = (dist_back + 3) % 4;}
                        17..=32 => {cur_val = ((dist_back - 17) / 8) + 8; extra_bit_len = 3; extra_bit_val = (dist_back + 7) % 8;}
                        33..=64 => {cur_val = ((dist_back - 33) / 16) + 10; extra_bit_len = 4; extra_bit_val = (dist_back + 15) % 16;}
                        65..=128 => {cur_val = ((dist_back - 65) / 32) + 12; extra_bit_len = 5; extra_bit_val = (dist_back + 31) % 32;}
                        129..=256 => {cur_val = ((dist_back - 129) / 64) + 14; extra_bit_len = 6; extra_bit_val = (dist_back + 63) % 64;}
                        257..=512 => {cur_val = ((dist_back - 257) / 128) + 16; extra_bit_len = 7; extra_bit_val = (dist_back + 127) % 128;}
                        513..=1024 => {cur_val = ((dist_back - 513) / 256) + 18; extra_bit_len = 8; extra_bit_val = (dist_back + 255) % 256;}
                        1025..=2048 => {cur_val = ((dist_back - 1025) / 512) + 20; extra_bit_len = 9; extra_bit_val = (dist_back + 511) % 512;}
                        2049..=4096 => {cur_val = ((dist_back - 2049) / 1024) + 22; extra_bit_len = 10; extra_bit_val = (dist_back + 1023) % 1024;}
                        4097..=8192 => {cur_val = ((dist_back - 4097) / 2048) + 24; extra_bit_len = 11; extra_bit_val = (dist_back + 2047) % 2048;}
                        8193..=16384 => {cur_val = ((dist_back - 8193) / 4096) + 26; extra_bit_len = 12; extra_bit_val = (dist_back + 4095) % 4096;}
                        16385..=32768 => {cur_val = ((dist_back - 16385) / 8192) + 28; extra_bit_len = 13; extra_bit_val = (dist_back + 8191) % 8192;}
                        _ => {}
                    }
                    let bits = usize_to_bits(cur_val, 5);
                    self.bit_buffer.extend(bits);
                    if extra_bit_len > 0 {
                        let mut extra_bits = usize_to_bits(extra_bit_val, extra_bit_len as u8);
                        extra_bits.reverse();
                        self.bit_buffer.extend(extra_bits);
                    }
                }
                _ => {
                    let mut cur_code = cur_val + BitStreamDeflator::MATCH_8_1_LOW - BitStreamDeflator::BASE_8_1_CODE;
                    let size: u8;
                    if BitStreamDeflator::MATCH_8_1_LOW <= cur_code && cur_code <= BitStreamDeflator::MATCH_8_1_HIGH {
                        size = 8;
                    }
                    else{
                        cur_code = cur_val + BitStreamDeflator::MATCH_9_LOW - BitStreamDeflator::BASE_9_CODE; 
                        size = 9;
                        
                    }
                    let bits = usize_to_bits(cur_code, size);
                    self.bit_buffer.extend(bits);
                }
            }
        }
        self.bit_buffer.extend(vec![false; 7]);  

        let mut bit_buffer_val: Vec<usize> = Vec::new();
        for i in self.bit_buffer.iter() {
            bit_buffer_val.push(*i as usize)
        }
        self.prep_look_back_buffer(); 
    }

    const MAX_LEN: usize = 258;

    fn lz77_baseline(&mut self){
        if self.data.len() > 1 {
            while self.data_pos < self.data.len()-2{
                let buffer = self.read_triplet();
                self.lz77map.entry(buffer.clone()).and_modify(|list| list.insert(0, self.data_pos)).or_insert(vec![self.data_pos]);
                let lookup = self.lz77map.get(&buffer).unwrap().clone();
                let mut curr_match = (buffer, 0, 0);
                if self.data_pos==26{
                    println!("lookup {:?}", lookup);
                }
                let curr_match_new = self.is_long_match(&lookup, curr_match.clone());
                if self.data_pos==26{
                    println!("curr_new_after_func {:?}", curr_match_new);
                }
                if (curr_match.1 != curr_match_new.1) && (curr_match.2 != curr_match_new.2){
                    curr_match = curr_match_new;
    
                    self.lz77_internal.push((self.data_pos, curr_match.1, curr_match.2));
    
                    self.lz77_output.push(60);
                    let first_arr = format!("{}", (curr_match.1));
                    let first_num = first_arr.as_bytes().clone();
                    for i in first_num.iter() {
                        self.lz77_output.push(*i as usize);
                    }
                    self.lz77_output.push(44);
                    let second_arr = format!("{}", (curr_match.2));
                    let second_num = second_arr.as_bytes().clone();
                    for i in second_num.iter() {
                        self.lz77_output.push(*i as usize);
                    }
                    self.lz77_output.push(62);
    
                    for i in self.data_pos + 1..=self.data_pos + curr_match.1 - 1 {
                        if i < self.data.len() - 2 {
                            let triplet = self.read_triplet_at(i);
                            self.lz77map.entry(triplet).and_modify(|list| list.insert(0, i)).or_insert(vec![i]); 
                        }
                    }
                    self.data_pos += curr_match.1 - 1;
                }
                else{
                    self.lz77_output.push(self.read_next_from(self.data_pos) as usize);
                }
                self.data_pos += 1;
            }
        }
        while self.data_pos < self.data.len() {
            self.lz77_output.push(self.read_next_from(self.data_pos) as usize);
            self.data_pos += 1;
        }
        self.data_pos = 0;
    }


    fn is_long_match(&mut self, lookup: &Vec<usize>, curr_match: (Vec<u8>, usize, usize)) -> (Vec<u8>, usize, usize){
        let mut curr_match_new = curr_match.clone();
        if lookup.len() > 1 {
            if self.data_pos - lookup[1] < 32768 {
                curr_match_new = (curr_match.0, 3, self.data_pos - lookup[1]);
            }
        }
        let mut out = curr_match_new.clone();
        for (i, idx) in lookup.iter().enumerate().skip(1){
            let mut prev = (Vec::new(), 1, 1);
            let mut buffer = curr_match_new.0.clone();
            if self.data_pos==26{
                println!("curr_match_new: {:?}, out: {:?}", curr_match_new, out);
            }
            while prev.1 != curr_match_new.1 {
                prev = curr_match_new.clone();
                if self.data_pos - idx < 32768 && buffer.len() < 258{
                    if self.data_pos + buffer.len() < self.data.len(){
                        buffer.push(self.read_next_from(self.data_pos + buffer.len()));
                        let comp = Vec::from(&self.data[*idx..(idx + buffer.len())]);
                        if buffer.eq(&comp) {
                            if buffer.len() > 258 {
                                println!("Literally how the fuck");
                            }
                            curr_match_new = (buffer.clone(), buffer.len(), self.data_pos - *idx);
                            out = curr_match_new.clone();
                        }
                        else {
                            curr_match_new = prev.clone();
                        }
                    }
                }
            }
            if self.data_pos==26{
                println!("curr_match_new: {:?}, out: {:?}", curr_match_new, out);
            }
        }
        return out;
    }
}

pub fn deflate(args: Vec<String>) {
    let file_name: String = args[1].to_string();
    let mut file = File::open(&file_name).expect("No File Found");
    let mut fdata: Vec<u8> = Vec::new();
    file.read_to_end(&mut fdata).expect("Unable to read data");
    let deflated_data: Vec<u8> = deflate_data(fdata);
    let mut new_name = file_name;
    
    new_name.push_str(".deflate");
    let mut new_file = File::create(&new_name).expect("Unable to create file");
    new_file.write_all(&deflated_data);
}

pub fn deflate_data(data: Vec<u8>) -> Vec<u8>{
    let mut bs: BitStreamDeflator = BitStreamDeflator::new(data);
    bs.write_huffman();
    return bs.look_back_buffer;
}

pub fn lz(args: Vec<String>){
    let file_name: String = args[1].to_string();
    let mut file = File::open(&file_name).expect("No File Found");
    let mut fdata: Vec<u8> = Vec::new();
    file.read_to_end(&mut fdata).expect("Unable to read data");
    let mut bs: BitStreamDeflator = BitStreamDeflator::new(fdata);
    bs.lz77_baseline();
    let mut lz_data: Vec<u8> = Vec::new();
    for i in bs.lz77_output.iter() {
        lz_data.push(*i as u8);
    }
    let mut new_name = file_name;
    new_name.push_str(".lz77");
    let mut new_file = File::create(&new_name).expect("Unable to create file");
    new_file.write_all(&lz_data);
}

pub fn deflate_with_77(args: Vec<String>) {
    let file_name: String = args[1].to_string();
    let mut file = File::open(&file_name).expect("No File Found");
    let mut fdata: Vec<u8> = Vec::new();
    file.read_to_end(&mut fdata).expect("Unable to read data");
    let deflated_data: Vec<u8> = deflate_data_with_77(fdata);
    
    let mut new_name = file_name;
    
    new_name.push_str(".deflate");
    let mut new_file = File::create(&new_name).expect("Unable to create file");
    new_file.write_all(&deflated_data);
}

pub fn deflate_data_with_77(data: Vec<u8>) -> Vec<u8>{
    let mut bs: BitStreamDeflator = BitStreamDeflator::new(data);
    bs.lz77_baseline();
    bs.write_huffman_with_lz77();
    return bs.look_back_buffer;
}