use binrw::{BinReaderExt, BinResult};
use slidetown::parsers::agt::{AgtReader, Entry, Header};
use std::io::Cursor;

#[test]
fn header_0_files() {
    let header_buffer: &[u8] = &[
        0x4E, 0x61, 0x79, 0x61, 0x50, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x51, 0x35, 0x19, 0x04, 0x06, 0x05, 0x02, 0x06, 0x47,
        0x2B, 0x0F,
    ];
    let mut header_cursor = Cursor::new(header_buffer);
    let header: Header = header_cursor.read_le().unwrap();
    assert_eq!(
        Header {
            what: 0,
            version: (1, 1),
            file_count: 0,
            what2: 422924551,
            what3: 33883652,
            what4: 254494470,
        },
        header
    );
}

#[test]
fn header_7_files() {
    let header_buffer: &[u8] = &[
        0x4E, 0x61, 0x79, 0x61, 0x50, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01,
        0x00, 0x07, 0x00, 0x00, 0x00, 0xAA, 0x5E, 0xC0, 0x7A, 0x04, 0x06, 0x05, 0x02, 0x06, 0x47,
        0x2B, 0x0F,
    ];
    let mut header_cursor = Cursor::new(header_buffer);
    let header: Header = header_cursor.read_le().unwrap();
    assert_eq!(
        Header {
            what: 0,
            version: (1, 1),
            file_count: 7,
            what2: 2059427498,
            what3: 33883652,
            what4: 254494470,
        },
        header
    );
}

#[test]
fn header_eof() {
    let header_buffer: &[u8] = &[
        0x4E, 0x61, 0x79, 0x61, 0x50, 0x61, 0x63, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01,
        0x00, 0x07, 0x00, 0x00, 0x00, 0xAA, 0x5E, 0xC0, 0x7A, 0x04, 0x06, 0x05, 0x02, 0x06, 0x47,
        0x2B, 0x0F,
    ];

    for i in 1..header_buffer.len() {
        let mut header_cursor = Cursor::new(&header_buffer[0..i]);
        let result: BinResult<Header> = header_cursor.read_le();
        assert_eq!(true, result.is_err());
    }
}

#[test]
fn entry_pivot() {
    let entry_buffer: &[u8] = &[
        0x1C, 0x82, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0xE1, 0x0C, 0x00, 0x00, 0x13, 0x00, 0x00,
        0x00, 0x44, 0x61, 0x74, 0x61, 0x5C, 0x62, 0x6C, 0x75, 0x65, 0x2D, 0x70, 0x69, 0x76, 0x6F,
        0x74, 0x2E, 0x6E, 0x69, 0x66,
    ];

    let mut entry_cursor = Cursor::new(entry_buffer);
    let entry: Entry = entry_cursor.read_le().unwrap();
    assert_eq!(
        Entry {
            header_offset: 164380,
            chunk_count: 1,
            decompressed_length: 3297,
            path: "Data\\blue-pivot.nif".to_string()
        },
        entry
    );
}

#[test]
fn dev_neodata() -> anyhow::Result<()> {
    let spooky_key: &[u8] = &[
        0x01, 0x05, 0x06, 0x02, 0x04, 0x03, 0x07, 0x08, 0x01, 0x05, 0x06, 0x0F, 0x04, 0x03, 0x07,
        0x0C, 0x31, 0x85, 0x76, 0x39, 0x34, 0x3D, 0x30, 0xE8, 0x67, 0x36, 0x36, 0x32, 0x3E, 0x33,
        0x34, 0x3B, 0x11, 0x15, 0x16, 0x16, 0x14, 0x13, 0x1D, 0x18, 0x11, 0x03, 0x06, 0x0C, 0x04,
        0x03, 0x06, 0x08, 0x2E, 0x55, 0x26, 0x23, 0x2A, 0x23, 0x2E, 0x28, 0x21, 0x21, 0x26, 0x27,
        0x2E, 0x00, 0x2D, 0x2D, 0xCF, 0xA5, 0x06, 0x02, 0x04, 0x0F, 0x07, 0x18, 0xE1, 0x15, 0x36,
        0x18, 0x60, 0x13, 0x1A, 0x19, 0x11, 0x15, 0x16, 0x10, 0x12, 0x13, 0x17, 0x38, 0xF1, 0x25,
    ];
    let agt_buffer = include_bytes!("../resources/fixtures/dev_neodata.agt");
    let mut agt_reader = AgtReader::new(Cursor::new(agt_buffer), spooky_key);

    let header: Header = Header::parse(&mut agt_reader).unwrap();
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

    let entries = Entry::parse_entries(&mut agt_reader, header.file_count as usize).unwrap();
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
        let data = agt_reader.read_entry(entry)?;
        assert_eq!(data.len(), entry.decompressed_length as usize);
    }

    Ok(())
}
