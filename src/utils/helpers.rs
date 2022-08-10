pub fn binary_to_dec(bin: &Vec<bool>) -> usize {
    let mut dec: usize = 0;

    for (i, bit) in bin.iter().enumerate() {
        if *bit {
            dec += 1 << bin.len() - i - 1
        }
    }
    return dec
}

pub fn usize_to_bits(num: usize, size: u8) -> Vec<bool>{
    let mut ret: Vec<bool> = Vec::new();
    for i in 0..size{
        ret.push(num & (1 << i) != 0);
    }
    ret.reverse();
    return ret
}