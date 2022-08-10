use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use super::helpers::*;

struct BitStreamInflator {
    data: Vec<u8>,
    byte_pos: usize,
    bit_pos: usize,
    cur_block_type: usize,
    look_back_buffer: Vec<u8>,
    finished: bool
}


impl BitStreamInflator {

    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            byte_pos: 0,
            bit_pos: 0,
            cur_block_type: 0,
            look_back_buffer: Vec::new(),
            finished: false
        }
    }

    fn get_next_bit(&mut self) -> bool{
        let ret = self.data[self.byte_pos] & (1 << self.bit_pos) != 0;
        self.bit_pos += 1;
        if self.bit_pos >= 8 {
            self.byte_pos += 1;
            self.bit_pos = 0;
        }
        return ret
    }

    fn read_ahead(&mut self, n: usize) -> Vec<bool> {
        let mut ret: Vec<bool> = Vec::new();
        for _ in 0..n {
            ret.push(self.get_next_bit());
        }
        return ret;
    }

    fn block_start(&mut self) {
        let bfinal = self.get_next_bit();
        if bfinal {
            self.finished = true;
        }
    }

    fn block_type(&mut self) {
        let mut type_bin = self.read_ahead(2);
        type_bin.reverse();
        self.cur_block_type = binary_to_dec(&type_bin) 
    }

    fn block_read(&mut self) {
        self.block_start();
        self.block_type();
        
        match self.cur_block_type {
            0 => {self.read_no_compression()}
            1 => {self.read_fixed_huffman()}
            2 => {self.read_dynamic_huffman()}
            _ => {}
        }
    }

    // TODO Add functionality if we want
    fn read_no_compression(&mut self) {}

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


    fn read_fixed_huffman(&mut self) {
        let mut code = 0;

        while code != BitStreamInflator::EOB_CODE {
            let mut cur_chunk = self.read_ahead(6);
            let mut found_match: bool = false;
            while !found_match {
                cur_chunk.push(self.get_next_bit());
                let cur_val = binary_to_dec(&cur_chunk);
                match cur_chunk.len() {
                    7 => {
                        if cur_val == BitStreamInflator::MATCH_7_LOW {
                            code = cur_val - BitStreamInflator::MATCH_7_LOW + BitStreamInflator::BASE_7_CODE;
                            found_match = true
                        }
                        // Length Case
                        if BitStreamInflator::MATCH_7_LOW < cur_val && cur_val <= BitStreamInflator::MATCH_7_HIGH {
                            code = cur_val - BitStreamInflator::MATCH_7_LOW + BitStreamInflator::BASE_7_CODE;
                            self.repeat_buffer(code);
                            found_match = true;
                        } 
                    }
                    8 => {
                        // Literal Case
                        if BitStreamInflator::MATCH_8_1_LOW <= cur_val && cur_val <= BitStreamInflator::MATCH_8_1_HIGH {
                            code = cur_val - BitStreamInflator::MATCH_8_1_LOW + BitStreamInflator::BASE_8_1_CODE;
                            self.write_literal_code(code);
                            found_match = true;
                        } 

                        // Length Case
                        else if BitStreamInflator::MATCH_8_2_LOW <= cur_val && cur_val <= BitStreamInflator::MATCH_8_2_HIGH {
                            code = cur_val - BitStreamInflator::MATCH_8_2_LOW + BitStreamInflator::BASE_8_2_CODE;
                            self.repeat_buffer(code);
                            found_match = true;
                        } 
                    }
                    9 => {
                        // Literal Case
                        if BitStreamInflator::MATCH_9_LOW <= cur_val && cur_val <= BitStreamInflator::MATCH_9_HIGH {
                            code = cur_val - BitStreamInflator::MATCH_9_LOW + BitStreamInflator::BASE_9_CODE;
                            self.write_literal_code(code);
                            found_match = true;
                        } 
                    }
                    _ => {}
                }
            }
        }
    }

    fn write_literal_code(&mut self, code: usize) {
        let c = code as u8;
        self.look_back_buffer.push(c);
    }
    
    fn write_literal_char(&mut self, c: u8) {
        self.look_back_buffer.push(c as u8);
    }

    const LENGTH_BASES: [usize; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];

    fn get_length(&mut self, code: usize) -> usize{
        let len: usize;
        let amount_to_read: usize;
        match code {
            265..=268 => {amount_to_read = 1;}
            269..=272 => {amount_to_read = 2;}
            273..=276 => {amount_to_read = 3;}
            277..=280 => {amount_to_read = 4;}
            281..=284 => {amount_to_read = 5;}
            _ => {amount_to_read = 0;}
        }
        let mut next_bits = self.read_ahead(amount_to_read);
        next_bits.reverse();
        let add_to_len = binary_to_dec(&next_bits);
        len = BitStreamInflator::LENGTH_BASES[(code - BitStreamInflator::EOB_CODE - 1)] + add_to_len;
        return len
    }

    fn repeat_buffer(&mut self, code: usize) {
        let len = self.get_length(code);
        let distance = self.get_distance();
        for _ in 0..len {
            self.write_literal_char(self.look_back_buffer[self.look_back_buffer.len() - distance])
        }
    }

    const REPEAT_BASES: [usize; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];

    fn get_distance(&mut self) -> usize{
        let distance: usize;
        let amount_to_read: usize;
        let distance_bits = self.read_ahead(5);
        let code = binary_to_dec(&distance_bits);
        match code{
            4..=5 => {amount_to_read = 1}
            6..=7 => {amount_to_read = 2}
            8..=9 => {amount_to_read = 3}
            10..=11 => {amount_to_read = 4}
            12..=13 => {amount_to_read = 5}
            14..=15 => {amount_to_read = 6}
            16..=17 => {amount_to_read = 7}
            18..=19 => {amount_to_read = 8}
            20..=21 => {amount_to_read = 9}
            22..=23 => {amount_to_read = 10}
            24..=25 => {amount_to_read = 11}
            26..=27 => {amount_to_read = 12}
            28..=29 => {amount_to_read = 13}
            _ => {amount_to_read = 0}
        }
        let mut next_bits = self.read_ahead(amount_to_read);
        next_bits.reverse();
        let add_to_distance = binary_to_dec(&next_bits);
        distance = BitStreamInflator::REPEAT_BASES[code] + add_to_distance;
        return distance;
    }

    const CODE_LEN_ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];

    fn read_dynamic_huffman(&mut self) {
        let mut bl_count_temp: HashMap<usize, usize> = HashMap::new();

        let mut hlitvec = self.read_ahead(5);
        hlitvec.reverse();
        let hlit = binary_to_dec(&hlitvec);
        let mut hdistvec = self.read_ahead(5);
        hdistvec.reverse();
        let hdist = binary_to_dec(&hdistvec);
        let mut hclenvec = self.read_ahead(4);
        hclenvec.reverse();
        let hclen = binary_to_dec(&hclenvec);

        let mut cl_bet: HashMap<usize, usize> = HashMap::new();
        for (i, num) in BitStreamInflator::CODE_LEN_ORDER.iter().enumerate(){
            if i < hclen + 4{
                let mut bits = self.read_ahead(3);
                bits.reverse();
                let code_len = binary_to_dec(&bits);
                cl_bet.insert(*num, code_len);
            }
        }

        let mut maxlen = 0;
        for (code, length) in cl_bet.iter(){
            bl_count_temp.entry(*length).and_modify(|count| *count += 1).or_insert(1);
            if length > &maxlen {
                maxlen = *length;
            } 
        }

        let mut bl_count: Vec<usize> = Vec::new();
        bl_count.push(0);
        for i in 1..=maxlen{
            match bl_count_temp.get_mut(&i){
                Some(v) => bl_count.push(*v),
                None => bl_count.push(0)
            }
        }

        let mut code: usize = 0;
        let mut next_code: Vec<usize> = Vec::new();
        let mut next_code_seen: Vec<usize> = Vec::new();
        next_code.push(0);
        for i in 1..=maxlen{
            code = (code + bl_count[i as usize - 1] as usize) << 1;
            next_code.push(code);
            next_code_seen.push(0);
        }

        let mut cl_map: HashMap<usize, HashMap<usize, usize>> = HashMap::new();

        for i in 0..=18 {
            if cl_bet.contains_key(&i) {
                let len = cl_bet.get(&i).unwrap();
                let new_code = next_code[*len as usize];
                if *len != 0{
                    next_code[*len as usize] += 1;
                }
                if !cl_map.contains_key(len) {
                    cl_map.insert(*len, HashMap::new());
                }
                cl_map.get_mut(len).unwrap().insert(new_code, i);
            }
        }

        //hlit decode
        let mut hlit_bet: HashMap<usize, usize> = HashMap::new();
        let mut max_read = hlit + 257;
        let mut count = 0;
        let mut prev: (usize, usize) = (0, 0);
        while count < max_read{
            let mut cur_chunk: Vec<bool> = Vec::new();
            let mut found_match = false;
            while !found_match {
                cur_chunk.push(self.get_next_bit());
                let mut cur_val = binary_to_dec(&cur_chunk);
                if cl_map.contains_key(&cur_chunk.len()) {
                    if cl_map.get(&cur_chunk.len()).unwrap().contains_key(&cur_val) {
                        let num = cl_map.get(&cur_chunk.len()).unwrap().get(&cur_val).unwrap();
                        let amount_to_read: usize;
                        match num{
                            16 => {
                                amount_to_read = 2;
                                let mut next_bits = self.read_ahead(amount_to_read);
                                next_bits.reverse();
                                let ahead = binary_to_dec(&next_bits) + 3;
                                for i in 0..ahead{
                                    if prev.0 <= 15 {
                                        hlit_bet.insert(count, prev.0);
                                        count += 1
                                    }
                                    if prev.0 == 17 || prev.0 == 18 {
                                        count += prev.1
                                    }
                                }
                            }
                            17 => {
                                amount_to_read = 3;
                                let mut next_bits = self.read_ahead(amount_to_read);
                                next_bits.reverse();
                                let ahead = binary_to_dec(&next_bits) + 3;
                                count += ahead;
                                prev = (*num as usize, ahead);
                            }
                            18 => {
                                amount_to_read = 7;
                                let mut next_bits = self.read_ahead(amount_to_read);
                                next_bits.reverse();
                                let ahead = binary_to_dec(&next_bits) + 11;
                                count += ahead;
                                prev = (*num as usize, ahead);
                            }
                            _ => {
                                hlit_bet.insert(count, *num as usize);
                                prev = (*num as usize, 0);
                                count += 1;
                            }
                        }
                        found_match = true;
                    }
                }
            }
        }

        let mut bl_count_temp: HashMap<usize, usize> = HashMap::new();
        let mut maxlen = 0;
        for (code, length) in hlit_bet.iter(){
            bl_count_temp.entry(*length).and_modify(|count| *count += 1).or_insert(1);
            if length > &maxlen {
                maxlen = *length;
            } 
        }

        let mut bl_count: Vec<usize> = Vec::new();
        bl_count.push(0);
        for i in 1..=maxlen{
            match bl_count_temp.get_mut(&i){
                Some(v) => bl_count.push(*v),
                None => bl_count.push(0)
            }
        }

        let mut code: usize = 0;
        let mut next_code: Vec<usize> = Vec::new();
        let mut next_code_seen: Vec<usize> = Vec::new();
        next_code.push(0);
        for i in 1..=maxlen{
            code = (code + bl_count[i as usize - 1] as usize) << 1;
            next_code.push(code);
            next_code_seen.push(0);
        }
        
        let mut hlit_map: HashMap<usize, HashMap<usize, usize>> = HashMap::new();

        for i in 0..=285{
            if i < hlit + 257{
                if hlit_bet.contains_key(&i) {
                    let len = hlit_bet.get(&i).unwrap();
                    let new_code = next_code[*len as usize];
                    if *len != 0{
                        next_code[*len as usize] += 1;
                    }
                    if !hlit_map.contains_key(len) {
                        hlit_map.insert(*len, HashMap::new());
                    }
                    hlit_map.get_mut(len).unwrap().insert(new_code, i);
                }
            }
        }

        let mut hdist_bet: HashMap<usize, usize> = HashMap::new();
        count = 0;
        max_read = hdist + 1;
        while count < max_read{
            let mut cur_chunk: Vec<bool> = Vec::new();
            let mut found_match = false;
            while !found_match {
                cur_chunk.push(self.get_next_bit());
                let mut cur_val = binary_to_dec(&cur_chunk);
                if cl_map.contains_key(&cur_chunk.len()) {
                    if cl_map.get(&cur_chunk.len()).unwrap().contains_key(&cur_val) {
                        let num = cl_map.get(&cur_chunk.len()).unwrap().get(&cur_val).unwrap();
                        let amount_to_read: usize;
                        match num{
                            16 => {
                                amount_to_read = 2;
                                let mut next_bits = self.read_ahead(amount_to_read);
                                next_bits.reverse();
                                let ahead = binary_to_dec(&next_bits) + 3;
                                for i in 0..ahead{
                                    if prev.0 <= 15 {
                                        hdist_bet.insert(count, prev.0);
                                        count += 1
                                    }
                                    if prev.0 == 17 || prev.0 == 18 {
                                        count += prev.1
                                    }
                                }
                            }
                            17 => {
                                amount_to_read = 3;
                                let mut next_bits = self.read_ahead(amount_to_read);
                                next_bits.reverse();
                                let ahead = binary_to_dec(&next_bits) + 3;
                                count += ahead;
                                prev = (*num as usize, ahead);
                            }
                            18 => {
                                amount_to_read = 7;
                                let mut next_bits = self.read_ahead(amount_to_read);
                                next_bits.reverse();
                                let ahead = binary_to_dec(&next_bits) + 11;
                                count += ahead;
                                prev = (*num as usize, ahead);
                            }
                            _ => {
                                hdist_bet.insert(count, *num as usize);
                                prev = (*num as usize, 0);
                                count += 1;
                            }
                        }
                        found_match = true;
                    }
                }
            }
        }
        
        let mut bl_count_temp: HashMap<usize, usize> = HashMap::new();
        let mut maxlen = 0;
        for (code, length) in hdist_bet.iter(){
            bl_count_temp.entry(*length).and_modify(|count| *count += 1).or_insert(1);
            if length > &maxlen {
                maxlen = *length;
            } 
        }

        let mut bl_count: Vec<usize> = Vec::new();
        bl_count.push(0);
        for i in 1..=maxlen{
            match bl_count_temp.get_mut(&i){
                Some(v) => bl_count.push(*v),
                None => bl_count.push(0)
            }
        }

        let mut code: usize = 0;
        let mut next_code: Vec<usize> = Vec::new();
        let mut next_code_seen: Vec<usize> = Vec::new();
        next_code.push(0);
        for i in 1..=maxlen{
            code = (code + bl_count[i as usize - 1] as usize) << 1;
            next_code.push(code);
            next_code_seen.push(0);
        }
        
        let mut hdist_map: HashMap<usize, HashMap<usize, usize>> = HashMap::new();

        for i in 0..=29{
            if i < hdist + 1{
                if hdist_bet.contains_key(&i) {
                    let len = hdist_bet.get(&i).unwrap();
                    let new_code = next_code[*len as usize];
                    if *len != 0{
                        next_code[*len as usize] += 1;
                    }
                    if !hdist_map.contains_key(len) {
                        hdist_map.insert(*len, HashMap::new());
                    }
                    hdist_map.get_mut(len).unwrap().insert(new_code, i);
                }
            }
        }

        let mut code = 0;
        

        while code != BitStreamInflator::EOB_CODE {
            let mut cur_chunk = Vec::new();
            let mut found_match: bool = false;
            while !found_match {
                cur_chunk.push(self.get_next_bit());
                let cur_val = binary_to_dec(&cur_chunk);
                if hlit_map.contains_key(&cur_chunk.len()) {
                    if hlit_map.get(&cur_chunk.len()).unwrap().contains_key(&cur_val) {
                        found_match = true;
                        code = *hlit_map.get(&cur_chunk.len()).unwrap().get(&cur_val).unwrap();
                        match code {
                            0..=255 => {
                                self.write_literal_code(code);
                            }
                            257..=285 => {
                                self.repeat_buffer_dynamic(code, &hdist_map);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }


    fn repeat_buffer_dynamic(&mut self, code: usize, hdist_map: &HashMap<usize, HashMap<usize, usize>>) {
        let len = self.get_length(code);
        let distance = self.get_distance_dynamic(hdist_map);
        for _ in 0..len {
            self.write_literal_char(self.look_back_buffer[self.look_back_buffer.len() - distance])
        }
    }

    fn get_distance_dynamic(&mut self, hdist_map: &HashMap<usize, HashMap<usize, usize>>) -> usize{
        let distance: usize;
        let amount_to_read: usize;
        let mut code: usize = 0;
        let mut found_match: bool = false;
        let mut cur_chunk = Vec::new();
        while !found_match {
            cur_chunk.push(self.get_next_bit());
            let cur_val = binary_to_dec(&cur_chunk);
            if hdist_map.contains_key(&cur_chunk.len()) {
                if hdist_map[&cur_chunk.len()].contains_key(&cur_val) {
                    found_match = true;
                    code = hdist_map[&cur_chunk.len()][&cur_val];
                }
            }
        }

        match code{
            4..=5 => {amount_to_read = 1}
            6..=7 => {amount_to_read = 2}
            8..=9 => {amount_to_read = 3}
            10..=11 => {amount_to_read = 4}
            12..=13 => {amount_to_read = 5}
            14..=15 => {amount_to_read = 6}
            16..=17 => {amount_to_read = 7}
            18..=19 => {amount_to_read = 8}
            20..=21 => {amount_to_read = 9}
            22..=23 => {amount_to_read = 10}
            24..=25 => {amount_to_read = 11}
            26..=27 => {amount_to_read = 12}
            28..=29 => {amount_to_read = 13}
            _ => {amount_to_read = 0}
        }
        let mut next_bits = self.read_ahead(amount_to_read);
        next_bits.reverse();
        let add_to_distance = binary_to_dec(&next_bits);
        distance = BitStreamInflator::REPEAT_BASES[code] + add_to_distance;
        return distance;
    }

    pub fn read(&mut self) {
        while !self.finished{
            self.block_read();
        }
    }
}

pub fn inflate(args: Vec<String>) {
    let file_name: &str = &args[1];
    let mut file = File::open(file_name).expect("No File Found");
    let mut fdata: Vec<u8> = Vec::new();
    file.read_to_end(&mut fdata).expect("Unable to read data");
    let inflated_data: Vec<u8> = inflate_data(fdata);

    let new_name = file_name.split(".deflate").into_iter().collect::<Vec<&str>>().join("");
    let mut new_file = File::create(&new_name).expect("Unable to create file");
    new_file.write_all(&inflated_data);
}

pub fn inflate_data(data: Vec<u8>) -> Vec<u8>{
    let mut bs: BitStreamInflator = BitStreamInflator::new(data);
    bs.read();
    return bs.look_back_buffer;
}

pub fn print_bitstream(args: Vec<String>) {
    let file_name: &str = &args[1];
    let mut file = File::open(file_name).expect("No File Found");
    let mut fdata: Vec<u8> = Vec::new();
    file.read_to_end(&mut fdata).expect("Unable to read data");
    let mut bs: BitStreamInflator = BitStreamInflator::new(fdata);
    let mut bits: Vec<usize> = Vec::new();
    while bs.byte_pos < bs.data.len() {
        bits.push(bs.get_next_bit() as usize);
    }
}
