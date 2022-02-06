use slidetown::parsers::loi::Loi;
use std::io::Cursor;

#[test]
fn header_mp_modeltable0() {
    let loi_buffer = include_bytes!("../resources/loi/dcr_mp_main_object0.loi");
    let mut loi_cursor = Cursor::new(loi_buffer);
    let loi: Loi = Loi::parse(&mut loi_cursor).unwrap();

    assert_eq!(20061222, loi.header.version_date);
    assert_eq!(3854, loi.header.block_count);
    assert_eq!(0, loi.blocks.first().unwrap().block_index);

    let block_1350 = &loi.blocks[1350];
    assert_eq!(1350, block_1350.block_index);
    assert_eq!(3, block_1350.object_count);

    let block_1350_obj_0 = &block_1350.objects[0];
    assert_eq!(
        (-386.99414, -2014.621, 39.298634),
        block_1350_obj_0.position
    );
    assert_eq!(
        ((1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)),
        block_1350_obj_0.rotation
    );
}
