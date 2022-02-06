use slidetown::parsers::lif::Lif;
use std::io::Cursor;

#[test]
fn header_mp_modeltable0() {
    let lif_buffer = include_bytes!("../resources/lif/dcr_mp_track1_terrain0.lif");
    let mut lif_cursor = Cursor::new(lif_buffer);
    let lif: Lif = Lif::parse(&mut lif_cursor).unwrap();

    assert_eq!(20061213, lif.header.version_date);
    assert_eq!(81, lif.header.block_count);

    assert_eq!(2741, lif.blocks.last().unwrap().index);
}
