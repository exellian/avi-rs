

#[cfg(test)]
mod tests {
    use avi_rs::bytes::{BigEndian, LittleEndian};

    #[test]
    fn byteorder() {
        let mut buf = [0u8;4];
        let u = 43608830 as u32;

        BigEndian::write_u32(u, &mut buf, 0);
        let r = BigEndian::read_u32(&buf, 0);

        println!("Write BigEndian: {:?}", buf);
        println!("Read BigEndian: {}", r);

        LittleEndian::write_u32(u, &mut buf, 0);
        let r1 = LittleEndian::read_u32(&buf, 0);

        println!("Write LittleEndian: {:?}", buf);
        println!("Read LittleEndian: {}", r1);

        assert_eq!(u, r);
        assert_eq!(u, r1);
    }

    #[test]
    fn parse_stream_index() {

        let buf = b"19";
        let str = std::str::from_utf8(buf).expect("Failed to parse Utf8!");
        let n: u8 = str.parse().expect("Not a number!");

        println!("The number is: {}", n);

        assert_eq!(n, 19);
    }

    #[test]
    fn signed() {
        let mut buf = [0u8;4];
        let i = -43608830;

        BigEndian::write_i32(i, &mut buf, 0);
        let r = BigEndian::read_i32(&buf, 0);

        println!("Write BigEndian: {:?}", buf);
        println!("Read BigEndian: {}", r);

        LittleEndian::write_i32(i, &mut buf, 0);
        let r1 = LittleEndian::read_i32(&buf, 0);

        println!("Write LittleEndian: {:?}", buf);
        println!("Read LittleEndian: {}", r1);

        assert_eq!(i, r);
        assert_eq!(i, r1);

    }
}

