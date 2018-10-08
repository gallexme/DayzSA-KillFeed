extern crate crc;
use self::crc::crc32;

#[derive(PartialEq, Debug)]
pub enum RconMessageType {
    Login = 0,
    Command = 1,
    Log = 2,
}

fn calc_crc(payload: &Vec<u8>) -> [u8; 4] {
   return unsafe { std::mem::transmute(crc32::checksum_ieee(payload.as_slice()).to_le()) };
}

fn create_header() -> Vec<u8> {
    vec![0x42, 0x45]
}

pub fn construct<'a>(command: RconMessageType, payload: Vec<u8>) -> Vec<u8> {
    let mut v = create_header();
    let mut pbv: Vec<u8> = vec![0xFF];
    pbv.push(command as u8);
    pbv.append(&mut Vec::from(payload));

    let crc: [u8; 4] = calc_crc(&pbv);
    v.push(crc[0]);
    v.push(crc[1]);
    v.push(crc[2]);
    v.push(crc[3]);

    v.append(&mut pbv);
    v
}