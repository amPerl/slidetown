use slidetown::parsers::nui::{
    Control, ControlInfo, ControlInfoAttributes, ControlNation, Document,
};

#[test]
fn legacy_nui() -> anyhow::Result<()> {
    let nui_buffer = include_bytes!("../resources/nui/legacy_game.nui");
    let nui_cursor = std::io::Cursor::new(nui_buffer);
    let nui = Document::from_read(nui_cursor)?;

    assert!(nui.component_list.is_some());
    assert_eq!(270, nui.dialog_list.dialogs.len());

    let first_dialog = nui.dialog_list.dialogs.first().unwrap();
    let first_control = first_dialog.controls.first().unwrap();
    assert_eq!(
        &Control {
            guid: Some("{E97B4F4F-A1F4-4DB5-B352-BF55A124E350}".to_string()),
            id: 10001,
            name: None,
            text_id: None,
            info: None,
            nation: None,
        },
        first_control
    );

    Ok(())
}

#[test]
fn modern_nui() -> anyhow::Result<()> {
    let nui_buffer = include_bytes!("../resources/nui/modern_game.nui");
    let nui_cursor = std::io::Cursor::new(nui_buffer);
    let nui = Document::from_read(nui_cursor)?;

    assert!(nui.component_list.is_none());
    assert_eq!(550, nui.dialog_list.dialogs.len());

    let first_dialog = nui.dialog_list.dialogs.first().unwrap();
    let first_control = first_dialog.controls.first().unwrap();
    assert_eq!(
        &Control {
            guid: None,
            id: 10001,
            name: None,
            text_id: None,
            info: Some(ControlInfo {
                kind: 1,
                name: Some("고정_BG".to_string()),
                attributes: ControlInfoAttributes {
                    attr_int_ec: Some(1),
                    sizes: [(1, (1024, 768))].iter().cloned().collect(),
                    positions: [(1, (0, 0))].iter().cloned().collect(),
                    uvs: [((1, 1), (0.0, 0.0, 1.0, 0.0, 0.0, 0.75, 1.0, 0.75))]
                        .iter()
                        .cloned()
                        .collect(),
                    images: [((1, 1), "0,0~1024,768_ui_loginbg.dds".to_string())]
                        .iter()
                        .cloned()
                        .collect(),

                    ..Default::default()
                }
            }),
            nation: Some(ControlNation {
                attr_int_c: 0,
                attr_int_f: 18,
                attr_tuple_h: (-512, -384),
                ..Default::default()
            })
        },
        first_control
    );

    Ok(())
}
