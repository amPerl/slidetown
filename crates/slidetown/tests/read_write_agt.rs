use slidetown::{
    agt::{AgtBuilder, AgtReader},
    parsers::agt::{Entry, Header},
};
use std::io::Cursor;

fn check_neodata(agt_buffer: &[u8], agt_key: &[u8]) -> anyhow::Result<Vec<(Entry, Vec<u8>)>> {
    let mut agt_file = Cursor::new(agt_buffer);
    let mut agt_reader = AgtReader::new(&mut agt_file, agt_key);

    let header: Header = agt_reader.read_header()?;
    assert_eq!(header.version, (1, 1,));
    assert_eq!(header.file_count, 5);

    let entries: Vec<Entry> = agt_reader.read_entries(header.file_count)?;
    assert_eq!(
        entries.iter().map(|e| e.path.clone()).collect::<Vec<_>>(),
        vec![
            "NeoData\\NC_chapter.xlt".to_string(),
            "NeoData\\NC_mission.xlt".to_string(),
            "NeoData\\NC_object.xlt".to_string(),
            "NeoData\\NC_objectDef.xlt".to_string(),
            "NeoData\\NC_quest.xlt".to_string(),
        ]
    );

    let mut entries_with_data = Vec::new();

    for entry in entries.into_iter() {
        let data = agt_reader.read_entry_data(&entry)?;
        assert_eq!(
            data.len(),
            entry.decompressed_length as usize,
            "entry length did not match specified length",
        );

        let data_u16 = data
            .chunks_exact(2)
            .map(|ch| u16::from_le_bytes(ch.try_into().unwrap()))
            .skip(1) // skip BOM
            .collect::<Vec<_>>();

        assert!(String::from_utf16(&data_u16).is_ok());

        entries_with_data.push((entry, data));
    }

    Ok(entries_with_data)
}

#[test]
fn dev_neodata() -> anyhow::Result<()> {
    let spooky_key: &[u8] = include_bytes!("../resources/agt/spooky_key.bin");
    let agt_buffer = include_bytes!("../resources/agt/dev_neodata.agt");

    // let mut agt_buffer_copy = agt_buffer.to_owned();
    // for (i, val) in agt_buffer_copy.iter_mut().enumerate().skip(32) {
    //     *val ^= spooky_key[i % spooky_key.len()];
    // }
    // std::fs::write("dev_neodata_plain.agt", &agt_buffer_copy)?;

    let entries_with_data = check_neodata(agt_buffer, spooky_key)?;

    let mut builder = AgtBuilder::new();
    for (entry, data) in entries_with_data.iter() {
        builder.add_entry_memory(entry.path.clone(), data);
    }

    let mut out_buf = Vec::new();
    let mut out_file = Cursor::new(&mut out_buf);
    builder.write(&mut out_file, spooky_key)?;

    // std::fs::write("dev_neodata_rewrite.agt", &out_buf)?;

    let new_entries_with_data = check_neodata(&out_buf, spooky_key)?;

    for ((_old_entry, old_data), (_new_entry, new_data)) in entries_with_data
        .into_iter()
        .zip(new_entries_with_data.into_iter())
    {
        assert_eq!(old_data, new_data);
    }

    Ok(())
}
