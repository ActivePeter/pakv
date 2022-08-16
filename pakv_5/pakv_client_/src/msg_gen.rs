use byteorder::{BigEndian, ByteOrder};

pub fn headlen_bytes(msgstr:&String) -> [u8; 4] {
    let mut a: [u8; 4] =[0,0,0,0];
    BigEndian::write_u32(&mut a,msgstr.len() as u32);

    a
}
