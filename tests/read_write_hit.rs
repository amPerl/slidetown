use slidetown::parsers::hit::Hit;
use std::io::Cursor;

#[test]
fn mp_track1_area_hit() {
    let in_buf = include_bytes!("../resources/hit/dcr_mp_track1_area.hit");
    let mut in_file = Cursor::new(in_buf);
    let hit: Hit = Hit::read(&mut in_file).unwrap();

    assert_eq!(381148, in_file.position());

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    hit.write(&mut out_file).unwrap();

    for (i, (in_byte, out_byte)) in in_buf.iter().zip(out_buf.iter()).enumerate() {
        assert_eq!(
            in_byte, out_byte,
            "Mismatching byte at {}, in: {:02X}, out: {:02X}",
            i, in_byte, out_byte
        );
    }
}
