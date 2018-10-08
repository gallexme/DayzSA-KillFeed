#[derive(PartialEq, Debug, Clone)]
pub enum RemotePacket {
    Login(bool),
    Command(u8, String),
    Log(u8, String),
    Unknown(Vec<u8>)
}

pub fn parse_packet(mut buf: Vec<u8>) -> RemotePacket {
    // interesting part starts
    // 2 byte ident, 4 byte crc, 1 byte FF, 1 byte command < we want this now
    assert!(buf.len() > 8);
    match buf[7] {
        0 => RemotePacket::Login(buf[8] != 0),
        1 => RemotePacket::Command(buf[8], String::from_utf8(buf.split_off(9)).unwrap()),
        2 => RemotePacket::Log(buf[8], String::from_utf8(buf.split_off(9)).unwrap()),
        _ => RemotePacket::Unknown(buf.split_off(8)),
    }
}