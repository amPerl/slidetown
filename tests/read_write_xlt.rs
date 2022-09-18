use std::{collections::HashMap, io::Cursor};

use slidetown::{
    parsers::xlt::{
        spoiler_list::SpoilerList,
        tire_list::TireList,
        vehicle_list::VehicleList,
        visual_item_list::{VisualItemCategory, VisualItemList},
        vshop_item_list::VShopItemList,
        Xlt,
    },
    xlt::InitConfiguration,
};

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
#[test]
fn visualitem_022() -> anyhow::Result<()> {
    let original_bytes = std::fs::read("resources/xlt/VisualItem_0.22.xlt")?;
    let xlt = Xlt::read(&mut Cursor::new(&original_bytes))?;

    assert_eq!(xlt.rows[6][5], "Silver");

    let visual_item_list = VisualItemList::from_xlt(&xlt)?;
    let kicker_front_1 = visual_item_list.entries.get(59).unwrap();
    assert_eq!(kicker_front_1.name, "Kicker front 1");
    assert_eq!(kicker_front_1.enabled_vehicle_id_indices.len(), 1);
    let vehicle_idx = kicker_front_1.enabled_vehicle_id_indices[0];
    let vehicle_name = &visual_item_list.vehicle_definitions.names[vehicle_idx];
    assert_eq!(vehicle_name, "Kicker");

    for entry in visual_item_list.entries.iter() {
        for idx in entry.enabled_vehicle_id_indices.iter() {
            assert!(visual_item_list.vehicle_definitions.ids.get(*idx).is_some());
        }
    }

    let mut rewritten_bytes = Vec::new();
    xlt.write(&mut Cursor::new(&mut rewritten_bytes))?;

    assert_eq!(&original_bytes, &rewritten_bytes[..original_bytes.len()]);

    Ok(())
}

#[test]
fn vshopitem_022() -> anyhow::Result<()> {
    let original_bytes = std::fs::read("resources/xlt/VShopItem_0.22.xlt")?;
    let xlt = Xlt::read(&mut Cursor::new(&original_bytes))?;

    assert_eq!(xlt.rows[3][3], "i_d_c_00001");

    let _vshop_item_list = VShopItemList::from_xlt(&xlt)?;

    let mut rewritten_bytes = Vec::new();
    xlt.write(&mut Cursor::new(&mut rewritten_bytes))?;

    assert_eq!(&original_bytes, &rewritten_bytes[..original_bytes.len()]);

    Ok(())
}

#[test]
fn tirelist_022() -> anyhow::Result<()> {
    let original_bytes = std::fs::read("resources/xlt/TireList_0.22.xlt")?;
    let xlt = Xlt::read(&mut Cursor::new(&original_bytes))?;

    assert_eq!(xlt.rows[1][1], "125");

    let tire_list = TireList::from_xlt(&xlt)?;

    assert_eq!(tire_list.entries[0].diameter, 180);

    let mut rewritten_bytes = Vec::new();
    xlt.write(&mut Cursor::new(&mut rewritten_bytes))?;

    assert_eq!(&original_bytes, &rewritten_bytes[..original_bytes.len()]);

    Ok(())
}

#[test]
fn spoilerlist_022() -> anyhow::Result<()> {
    let original_bytes = std::fs::read("resources/xlt/SpoilerList_0.22.xlt")?;
    let xlt = Xlt::read(&mut Cursor::new(&original_bytes))?;

    assert_eq!(xlt.rows[1][1], "34");

    let spoiler_list = SpoilerList::from_xlt(&xlt)?;

    assert_eq!(spoiler_list.entries[0].id, 32);

    let mut rewritten_bytes = Vec::new();
    xlt.write(&mut Cursor::new(&mut rewritten_bytes))?;

    assert_eq!(&original_bytes, &rewritten_bytes[..original_bytes.len()]);

    Ok(())
}

#[test]
fn init_configuration() -> anyhow::Result<()> {
    let vehicle_list_bytes = std::fs::read("resources/xlt/VehicleList_0.22.xlt")?;
    let vehicle_list_xlt = Xlt::read(&mut Cursor::new(&vehicle_list_bytes))?;
    let visual_item_list_bytes = std::fs::read("resources/xlt/VisualItem_0.22.xlt")?;
    let visual_item_list_xlt = Xlt::read(&mut Cursor::new(&visual_item_list_bytes))?;
    let vshop_item_list_bytes = std::fs::read("resources/xlt/VShopItem_0.22.xlt")?;
    let vshop_item_list_xlt = Xlt::read(&mut Cursor::new(&vshop_item_list_bytes))?;
    let tire_list_bytes = std::fs::read("resources/xlt/TireList_0.22.xlt")?;
    let tire_list_xlt = Xlt::read(&mut Cursor::new(&tire_list_bytes))?;
    let spoiler_list_bytes = std::fs::read("resources/xlt/SpoilerList_0.22.xlt")?;
    let spoiler_list_xlt = Xlt::read(&mut Cursor::new(&spoiler_list_bytes))?;

    let init = InitConfiguration::from_xlts(
        &vehicle_list_xlt,
        &visual_item_list_xlt,
        &vshop_item_list_xlt,
        &tire_list_xlt,
        &spoiler_list_xlt,
    )?;

    for vehicle in init.player_vehicles() {
        // eprintln!(
        //     "id {:?} name {:?} file {:?}",
        //     vehicle.id, vehicle.name, vehicle.file_name
        // );

        let defaults = init.vehicle_default_items(vehicle.id);

        if let Some(tire) = defaults.get(&VisualItemCategory::Tire) {
            let _tire_info = init.tire_info(tire.category_item_id);
            // eprintln!("\ttire info {:?}", tire_info);
        }

        if let Some(spoiler) = defaults.get(&VisualItemCategory::Spoiler) {
            // eprintln!(
            //     "\t\tspoiler id {:?} category item id {:?}",
            //     spoiler.id, spoiler.category_item_id
            // );
            let _spoiler_info = init.spoiler_info(spoiler.category_item_id);
            // eprintln!("\tspoiler info {:?}", spoiler_info);
        }

        // for item in defaults.values() {
        //     eprintln!("\tname {:?} id {:?}", item.name, item.item_id);
        // }
    }

    Ok(())
}
