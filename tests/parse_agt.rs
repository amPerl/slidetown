use slidetown::{
    agt::AgtReader,
    parsers::agt::{Entry, Header},
};
use std::io::Cursor;

#[test]
fn dev_neodata() -> anyhow::Result<()> {
    let spooky_key: &[u8] = include_bytes!("../resources/agt/spooky_key.bin");
    let agt_buffer = include_bytes!("../resources/agt/dev_neodata.agt");

    let mut agt_reader = AgtReader::new(Cursor::new(agt_buffer), spooky_key);

    let header: Header = agt_reader.read_header().unwrap();
    assert_eq!(
        header,
        Header {
            what: 0,
            version: (1, 1,),
            file_count: 5,
            what2: 2191538612,
            what3: 33883652,
            what4: 254494470,
        }
    );

    let entries: Vec<Entry> = agt_reader.read_entries(header.file_count).unwrap();
    assert_eq!(
        entries,
        vec![
            Entry {
                header_offset: 221,
                chunk_count: 1,
                decompressed_length: 5454,
                path: "NeoData\\NC_chapter.xlt".to_string(),
            },
            Entry {
                header_offset: 1945,
                chunk_count: 20,
                decompressed_length: 318218,
                path: "NeoData\\NC_mission.xlt".to_string(),
            },
            Entry {
                header_offset: 56241,
                chunk_count: 8,
                decompressed_length: 117256,
                path: "NeoData\\NC_object.xlt".to_string(),
            },
            Entry {
                header_offset: 69988,
                chunk_count: 1,
                decompressed_length: 1944,
                path: "NeoData\\NC_objectDef.xlt".to_string(),
            },
            Entry {
                header_offset: 70602,
                chunk_count: 2,
                decompressed_length: 29016,
                path: "NeoData\\NC_quest.xlt".to_string(),
            },
        ]
    );

    for entry in entries.iter() {
        let data = agt_reader.read_entry_data(entry)?;
        assert_eq!(data.len(), entry.decompressed_length as usize);

        let data_u16 = data
            .chunks_exact(2)
            .map(|ch| u16::from_le_bytes(ch.try_into().unwrap()))
            .skip(1) // skip BOM
            .collect::<Vec<_>>();

        assert!(String::from_utf16(&data_u16).is_ok());

        // if let Ok(data_str) = String::from_utf16(&data_u16) {
        //     for line in data_str.split("\r\n") {
        //         if !line.is_empty() {
        //             eprintln!("{:?}", line.split('\t').collect::<Vec<_>>());
        //         }
        //     }
        // }
    }

    Ok(())
}
