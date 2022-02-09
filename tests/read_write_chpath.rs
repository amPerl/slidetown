use slidetown::parsers::chpath::Chpath;
use std::io::Cursor;

#[test]
fn oros_track1_chpath() {
    let in_buf = include_bytes!("../resources/chpath/PVP_map3_01.chpath");
    let mut in_file = Cursor::new(in_buf);
    let chpath = Chpath::read(&mut in_file).unwrap();

    assert_eq!(9369, in_file.position());

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    chpath.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}
