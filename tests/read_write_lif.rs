use slidetown::parsers::lif::Lif;
use std::io::Cursor;

#[test]
fn mp_track1_terrain0_lif() {
    let in_buf = include_bytes!("../resources/lif/dcr_mp_track1_terrain0.lif");
    let mut in_file = Cursor::new(in_buf);
    let lif = Lif::read(&mut in_file).unwrap();

    assert_eq!(668, in_file.position());

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    lif.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}