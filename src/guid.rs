// Copied and adapted from windows-rs (src/core/guid.rs)
use windows::core::GUID;

trait HexReader {
    fn next_u8(&mut self) -> u8;
    fn next_u16(&mut self) -> u16;
    fn next_u32(&mut self) -> u32;
}

impl HexReader for std::str::Bytes<'_> {
    fn next_u8(&mut self) -> u8 {
        let value = self.next().unwrap();
        match value {
            b'0'..=b'9' => value - b'0',
            b'A'..=b'F' => 10 + value - b'A',
            b'a'..=b'f' => 10 + value - b'a',
            _ => panic!(),
        }
    }

    fn next_u16(&mut self) -> u16 {
        self.next_u8().into()
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u8().into()
    }
}

pub trait TryParse<T> {
    fn try_parse(&self) -> Option<T>;
}

impl TryParse<GUID> for &str {
    fn try_parse(&self) -> Option<GUID> {
        if self.len() == 36 {
            let mut bytes = self.bytes();

            let a = ((bytes.next_u32() * 16 + bytes.next_u32()) << 24)
                + ((bytes.next_u32() * 16 + bytes.next_u32()) << 16)
                + ((bytes.next_u32() * 16 + bytes.next_u32()) << 8)
                + bytes.next_u32() * 16
                + bytes.next_u32();
            if bytes.next().unwrap() == b'-' {
                let b = ((bytes.next_u16() * 16 + (bytes.next_u16())) << 8)
                    + bytes.next_u16() * 16
                    + bytes.next_u16();
                if bytes.next().unwrap() == b'-' {
                    let c = ((bytes.next_u16() * 16 + bytes.next_u16()) << 8)
                        + bytes.next_u16() * 16
                        + bytes.next_u16();
                    if bytes.next().unwrap() == b'-' {
                        let d = bytes.next_u8() * 16 + bytes.next_u8();
                        let e = bytes.next_u8() * 16 + bytes.next_u8();
                        if bytes.next().unwrap() == b'-' {
                            let f = bytes.next_u8() * 16 + bytes.next_u8();
                            let g = bytes.next_u8() * 16 + bytes.next_u8();
                            let h = bytes.next_u8() * 16 + bytes.next_u8();
                            let i = bytes.next_u8() * 16 + bytes.next_u8();
                            let j = bytes.next_u8() * 16 + bytes.next_u8();
                            let k = bytes.next_u8() * 16 + bytes.next_u8();

                            Some(GUID::from_values(a, b, c, [d, e, f, g, h, i, j, k]))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}
