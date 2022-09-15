use std::io::Cursor;

use slidetown::parsers::xlt::{VehicleList, Xlt};

#[test]
fn vehiclelist_022() -> anyhow::Result<()> {
    let original_bytes = std::fs::read("resources/xlt/VehicleList_0.22.xlt")?;
    let xlt = Xlt::read(&mut Cursor::new(&original_bytes))?;

    assert_eq!(xlt.rows[0][7], "트래픽카 Cnt");

    let _vehicle_list = VehicleList::from_xlt(&xlt)?;
    // dbg!(&vehicle_list.meta);
    // for entry in vehicle_list.entries.iter() {
    //     eprintln!("{:?}", entry);
    // }

    let mut rewritten_bytes = Vec::new();
    xlt.write(&mut Cursor::new(&mut rewritten_bytes))?;

    assert_eq!(&original_bytes, &rewritten_bytes[..original_bytes.len()]);

    Ok(())
}
