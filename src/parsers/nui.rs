use anyhow::Context;
use quick_xml::{
    events::{attributes::Attribute, BytesStart, Event},
    Reader,
};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

#[derive(Debug)]
pub struct Document {
    pub file_info: FileInfo,
    pub dialog_list: DialogList,
    pub component_list: Option<ComponentList>,
}

impl Document {
    pub fn from_read<R>(read: R) -> anyhow::Result<Self>
    where
        R: Read,
    {
        let buf_reader = BufReader::new(read);
        let mut reader = Reader::from_reader(buf_reader);
        let mut buf = Vec::new();

        let mut file_info = None;
        let mut dialog_list = None;
        let mut component_list = None;

        loop {
            let event = reader.read_event(&mut buf)?;
            match event {
                Event::Start(bytes_start) => {
                    let name = bytes_start.name();

                    if name == b"DIALOGLIST" {
                        dialog_list = Some(DialogList::from_event(bytes_start, &mut reader)?);
                    } else if name == b"COMPONENTLIST" {
                        component_list = Some(ComponentList::from_event(bytes_start, &mut reader)?);
                    }
                }
                Event::End(bytes_end) => {
                    let name = bytes_end.name();
                    if name == b"document" {
                        break;
                    }
                }
                Event::Empty(bytes_start) => {
                    let name = bytes_start.name();

                    if name == b"FILEINFO" {
                        file_info = Some(FileInfo::from_event(bytes_start, &mut reader)?);
                    }
                }
                Event::Eof => {
                    break;
                }
                _ => {}
            }
        }

        Ok(Self {
            file_info: file_info.context("Missing FILEINFO")?,
            dialog_list: dialog_list.context("Missing DIALOGLIST")?,
            component_list,
        })
    }
}

#[derive(Debug)]
pub struct FileInfo {
    pub nation: Option<usize>,
    pub time: String,
    pub kind: String,
}

impl FileInfo {
    pub fn from_event<T>(bytes_start: BytesStart, reader: &mut Reader<T>) -> anyhow::Result<Self>
    where
        T: BufRead,
    {
        let mut nation = None;
        let mut time = None;
        let mut kind = None;

        for attribute in bytes_start.attributes() {
            let attribute = attribute?;
            if attribute.key == b"Time" {
                time = Some(attribute.unescape_and_decode_value(reader)?)
            } else if attribute.key == b"Type" {
                kind = Some(attribute.unescape_and_decode_value(reader)?)
            } else if attribute.key == b"Nation" {
                nation = Some(attribute.unescape_and_decode_value(reader)?.parse()?)
            } else {
                println!(
                    "FileInfo ignoring attribute: {:?}",
                    String::from_utf8_lossy(attribute.key)
                );
            }
        }

        Ok(Self {
            nation,
            kind: kind.context("Missnig Type")?,
            time: time.context("Missing Time")?,
        })
    }
}

#[derive(Debug)]
pub struct DialogList {
    pub dialogs: Vec<Dialog>,
}

impl DialogList {
    pub fn from_event<T>(_bytes_start: BytesStart, reader: &mut Reader<T>) -> anyhow::Result<Self>
    where
        T: BufRead,
    {
        let mut buf = Vec::new();
        let mut dialogs = Vec::new();

        loop {
            let event = reader.read_event(&mut buf)?;
            match event {
                Event::Start(bytes_start) => {
                    let name = bytes_start.name();
                    if name == b"DIALOG" || name == b"D" {
                        dialogs.push(Dialog::from_event(bytes_start, reader)?);
                    }
                }
                Event::End(bytes_end) => {
                    if bytes_end.name() == b"DIALOGLIST" {
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(Self { dialogs })
    }
}

#[derive(Debug)]
pub struct ComponentList {}

impl ComponentList {
    pub fn from_event<T>(_bytes_start: BytesStart, reader: &mut Reader<T>) -> anyhow::Result<Self>
    where
        T: BufRead,
    {
        let mut buf = Vec::new();

        loop {
            let event = reader.read_event(&mut buf)?;
            if let Event::End(bytes_end) = event {
                let name = bytes_end.name();
                if name == b"COMPONENTLIST" {
                    break;
                }
            }
        }

        Ok(Self {})
    }
}

#[derive(Debug, PartialEq)]
pub struct Dialog {
    pub name: Option<String>,
    pub guid: Option<String>,
    pub id: usize,
    pub text_id: Option<String>,
    pub alignment_mask: Option<u8>,
    pub attr_tuple_f: Option<(isize, isize)>,
    pub offset: Option<(isize, isize)>,
    pub attr_tuple_h: Option<(isize, isize)>,
    pub attr_boolish_i: Option<u8>,
    pub controls: Vec<Control>,
}

impl Dialog {
    pub fn from_event<T>(bytes_start: BytesStart, reader: &mut Reader<T>) -> anyhow::Result<Self>
    where
        T: BufRead,
    {
        let mut buf = Vec::new();

        let mut name = None;
        let mut guid = None;
        let mut id = None;
        let mut text_id = None;
        let mut alignment_mask = None;
        let mut attr_tuple_f = None;
        let mut offset = None;
        let mut attr_tuple_h = None;
        let mut attr_boolish_i = None;

        for attribute in bytes_start.attributes() {
            let attribute = attribute?;
            if attribute.key == b"A" {
                name = Some(attribute.unescape_and_decode_value(reader)?)
            } else if attribute.key == b"B" {
                guid = Some(attribute.unescape_and_decode_value(reader)?)
            } else if attribute.key == b"C" {
                id = Some(attribute.unescape_and_decode_value(reader)?.parse()?)
            } else if attribute.key == b"D" {
                text_id = Some(attribute.unescape_and_decode_value(reader)?)
            } else if attribute.key == b"E" {
                alignment_mask = Some(attribute.unescape_and_decode_value(reader)?.parse()?)
            } else if attribute.key == b"F" {
                attr_tuple_f = Some(parse_tuple(&attribute, reader)?);
            } else if attribute.key == b"G" {
                offset = Some(parse_tuple(&attribute, reader)?);
            } else if attribute.key == b"H" {
                attr_tuple_h = Some(parse_tuple(&attribute, reader)?);
            } else if attribute.key == b"I" {
                attr_boolish_i = Some(attribute.unescape_and_decode_value(reader)?.parse()?);
            } else {
                println!(
                    "Dialog ignoring attribute: {:?}",
                    String::from_utf8_lossy(attribute.key)
                );
            }
        }

        let mut controls = Vec::new();

        loop {
            let event = reader.read_event(&mut buf)?;
            match event {
                Event::Start(bytes_start) => {
                    let name = bytes_start.name();
                    if name == b"CONTROL" || name == b"C" {
                        controls.push(Control::from_event(bytes_start, reader)?);
                    }
                }
                Event::End(bytes_end) => {
                    let name = bytes_end.name();
                    if name == b"DIALOG" || name == b"D" {
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            name,
            guid,
            id: id.context("Missing C")?,
            text_id,
            alignment_mask,
            attr_tuple_f,
            offset,
            attr_tuple_h,
            attr_boolish_i,
            controls,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Control {
    pub guid: Option<String>,
    pub id: usize,
    pub name: Option<String>,
    pub text_id: Option<String>,
    pub info: Option<ControlInfo>,
    pub nation: Option<ControlNation>,
}

impl Control {
    pub fn from_event<T>(bytes_start: BytesStart, reader: &mut Reader<T>) -> anyhow::Result<Self>
    where
        T: BufRead,
    {
        let mut buf = Vec::new();

        let mut name = None;
        let mut guid = None;
        let mut text_id = None;
        let mut id = None;
        let mut info = None;
        let mut nation = None;

        for attribute in bytes_start.attributes() {
            let attribute = attribute?;
            if attribute.key == b"A" {
                guid = Some(attribute.unescape_and_decode_value(reader)?);
            } else if attribute.key == b"B" {
                id = Some(attribute.unescape_and_decode_value(reader)?.parse()?);
            } else if attribute.key == b"D" {
                name = Some(attribute.unescape_and_decode_value(reader)?);
            } else if attribute.key == b"E" {
                text_id = Some(attribute.unescape_and_decode_value(reader)?);
            } else {
                println!(
                    "Control ignoring attribute: {:?}",
                    String::from_utf8_lossy(attribute.key)
                );
            }
        }

        loop {
            let event = reader.read_event(&mut buf)?;
            match event {
                Event::Start(bytes_start) => {
                    let name = bytes_start.name();
                    if name == b"INFO" {
                        info = Some(ControlInfo::from_event(bytes_start, reader, false)?);
                    } else if name == b"NATION" {
                        nation = Some(ControlNation::from_event(bytes_start, reader, false)?);
                    }
                }
                Event::Empty(bytes_start) => {
                    let name = bytes_start.name();
                    if name == b"INFO" {
                        info = Some(ControlInfo::from_event(bytes_start, reader, true)?);
                    } else if name == b"NATION" {
                        nation = Some(ControlNation::from_event(bytes_start, reader, true)?);
                    }
                }
                Event::End(bytes_end) => {
                    let name = bytes_end.name();
                    if name == b"CONTROL" || name == b"C" {
                        break;
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            guid,
            id: id.context("Missing B")?,
            text_id,
            name,
            info,
            nation,
        })
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct ControlInfo {
    pub kind: usize,
    pub name: Option<String>,
    pub attributes: ControlInfoAttributes,
}

impl ControlInfo {
    pub fn from_event<T>(
        bytes_start: BytesStart,
        reader: &mut Reader<T>,
        empty: bool,
    ) -> anyhow::Result<Self>
    where
        T: BufRead,
    {
        let mut buf = Vec::new();

        let mut kind = None;
        let mut name = None;

        // let decoded_bytes_start = bytes_start.unescape_and_decode(&mut reader)?;
        // eprintln!("{}", decoded_bytes_start);

        for attribute in bytes_start.attributes() {
            let attribute = attribute?;
            if attribute.key == b"TYPE" {
                kind = Some(attribute.unescape_and_decode_value(reader)?.parse()?)
            } else if attribute.key == b"NM" {
                name = Some(attribute.unescape_and_decode_value(reader)?)
            }
        }

        let attributes = ControlInfoAttributes::from_bytes_start(bytes_start, reader)?;

        if !empty {
            loop {
                let event = reader.read_event(&mut buf)?;
                if let Event::End(bytes_end) = event {
                    let name = bytes_end.name();
                    if name == b"INFO" {
                        break;
                    }
                }
            }
        }

        Ok(Self {
            kind: kind.context("Missing Type")?,
            name,
            attributes,
        })
    }
}

pub type Vec2usize = (usize, usize);
pub type Vec2isize = (isize, isize);
pub type Vec8f = (f64, f64, f64, f64, f64, f64, f64, f64);

#[derive(Debug, PartialEq, Default)]
pub struct ControlInfoAttributes {
    pub attr_int_ec: Option<usize>,
    pub sizes: HashMap<usize, Vec2usize>,
    pub positions: HashMap<usize, (isize, isize)>,
    pub uvs: HashMap<Vec2usize, Vec8f>,
    pub images: HashMap<Vec2usize, String>,
    pub colors: HashMap<Vec2usize, String>,
    pub texts: HashMap<usize, String>,
    pub text_aligns: HashMap<usize, usize>,
    pub fonts: HashMap<usize, String>,
    pub font_colors: HashMap<Vec2usize, String>,
    pub attr_boolish_map_fo: HashMap<Vec2usize, u8>,
    pub attr_color_map_foc: HashMap<Vec2usize, String>,
    pub attr_int_li: Option<isize>,
    pub attr_int_f1: Option<isize>,
    pub attr_int_tl: Option<String>,
    pub attr_boolish_sv: Option<u8>,
    pub sounds: HashMap<Vec2usize, String>,
}

impl ControlInfoAttributes {
    pub fn from_bytes_start<T>(
        bytes_start: BytesStart,
        reader: &mut Reader<T>,
    ) -> anyhow::Result<Self>
    where
        T: BufRead,
    {
        let mut attr_int_ec = None;
        let mut sizes = HashMap::new();
        let mut positions = HashMap::new();
        let mut uvs = HashMap::new();
        let mut images = HashMap::new();
        let mut colors = HashMap::new();
        let mut texts = HashMap::new();
        let mut text_aligns = HashMap::new();
        let mut fonts = HashMap::new();
        let mut font_colors = HashMap::new();
        let mut attr_boolish_map_fo = HashMap::new();
        let mut attr_color_map_foc = HashMap::new();
        let mut attr_int_li = None;
        let mut attr_int_f1 = None;
        let mut attr_int_tl = None;
        let mut attr_boolish_sv = None;
        let mut sounds = HashMap::new();

        for attribute in bytes_start.attributes() {
            let mut consumed = false;
            let attribute = attribute?;

            if attribute.key == b"TYPE"
                || attribute.key == b"NM"
                || attribute.key == [199, 209, 185, 248]
                || attribute.key == [188, 246, 193, 164]
            {
                continue;
            }

            if attribute.key == b"EC" {
                attr_int_ec = Some(attribute.unescape_and_decode_value(reader)?.parse()?);
                consumed = true;
            } else if attribute.key == b"LI" {
                attr_int_li = Some(attribute.unescape_and_decode_value(reader)?.parse()?);
                consumed = true;
            } else if attribute.key == b"F1" {
                attr_int_f1 = Some(attribute.unescape_and_decode_value(reader)?.parse()?);
                consumed = true;
            } else if attribute.key == b"TL" {
                attr_int_tl = Some(attribute.unescape_and_decode_value(reader)?);
                consumed = true;
            } else if attribute.key == b"SV" {
                attr_boolish_sv = Some(attribute.unescape_and_decode_value(reader)?.parse()?);
                consumed = true;
            }

            if !consumed && attribute.key.len() > 3 {
                let head = &attribute.key[0..3];
                let tail = &attribute.key[3..];

                if head == b"POS" {
                    let map_key = parse_index(tail)?;
                    positions.insert(map_key, parse_tuple(&attribute, reader)?);
                    consumed = true;
                } else if head == b"FOC" {
                    let map_key = parse_indices(tail)?;
                    attr_color_map_foc
                        .insert(map_key, attribute.unescape_and_decode_value(reader)?);
                    consumed = true;
                }
            }

            if !consumed && attribute.key.len() > 2 {
                let head = &attribute.key[0..2];
                let tail = &attribute.key[2..];

                if head == b"SZ" {
                    let map_key = parse_index(tail)?;
                    sizes.insert(map_key, parse_tuple(&attribute, reader)?);
                    consumed = true;
                } else if head == b"UV" {
                    let map_key = parse_indices(tail)?;
                    uvs.insert(map_key, parse_uv(&attribute, reader)?);
                    consumed = true;
                } else if head == b"TA" {
                    let map_key = parse_index(tail)?;
                    text_aligns.insert(
                        map_key,
                        attribute.unescape_and_decode_value(reader)?.parse()?,
                    );
                    consumed = true;
                } else if head == b"FN" {
                    let map_key = parse_index(tail)?;
                    fonts.insert(map_key, attribute.unescape_and_decode_value(reader)?);
                    consumed = true;
                } else if head == b"FC" {
                    let map_key = parse_indices(tail)?;
                    font_colors.insert(map_key, attribute.unescape_and_decode_value(reader)?);
                    consumed = true;
                } else if head == b"FO" {
                    let map_key = parse_indices(tail)?;
                    attr_boolish_map_fo.insert(
                        map_key,
                        attribute.unescape_and_decode_value(reader)?.parse()?,
                    );
                    consumed = true;
                }
            }

            if !consumed && attribute.key.len() > 1 {
                let head = &attribute.key[0..1];
                let tail = &attribute.key[1..];

                if head == b"I" {
                    let map_key = parse_indices(tail)?;
                    images.insert(map_key, attribute.unescape_and_decode_value(reader)?);
                    consumed = true;
                } else if head == b"C" {
                    let map_key = parse_indices(tail)?;
                    colors.insert(map_key, attribute.unescape_and_decode_value(reader)?);
                    consumed = true;
                } else if head == b"T" {
                    let map_key = parse_index(tail)?;
                    texts.insert(map_key, attribute.unescape_and_decode_value(reader)?);
                    consumed = true;
                } else if head == b"S" {
                    let map_key = parse_indices(tail)?;
                    sounds.insert(map_key, attribute.unescape_and_decode_value(reader)?);
                    consumed = true;
                }
            }

            if !consumed {
                anyhow::bail!(
                    "ControlInfoAttributes found unknown attribute: {:?} {:?}",
                    String::from_utf8_lossy(attribute.key),
                    attribute.key
                );
            }
        }

        Ok(Self {
            attr_int_ec,
            sizes,
            positions,
            uvs,
            images,
            colors,
            texts,
            text_aligns,
            fonts,
            font_colors,
            attr_boolish_map_fo,
            attr_color_map_foc,
            attr_int_li,
            attr_int_f1,
            attr_int_tl,
            attr_boolish_sv,
            sounds,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct ControlNation {
    pub attr_int_c: usize,
    pub attr_int_f: usize,
    pub attr_int_g: usize,
    pub attr_tuple_h: (isize, isize),
    pub attr_float_i: f64,
    pub attr_int_j: usize,
    pub attr_int_k: usize,
}

impl Default for ControlNation {
    fn default() -> Self {
        Self {
            attr_int_c: 0,
            attr_int_f: 9,
            attr_int_g: 1,
            attr_tuple_h: (0, 0),
            attr_float_i: 0.0,
            attr_int_j: 0,
            attr_int_k: 1,
        }
    }
}

impl ControlNation {
    pub fn from_event<T>(
        bytes_start: BytesStart,
        reader: &mut Reader<T>,
        empty: bool,
    ) -> anyhow::Result<Self>
    where
        T: BufRead,
    {
        let mut buf = Vec::new();

        let mut this = Self::default();

        // let decoded_bytes_start = bytes_start.unescape_and_decode(&mut reader)?;
        // eprintln!("{}", decoded_bytes_start);

        for attribute in bytes_start.attributes() {
            let attribute = attribute?;
            if attribute.key == b"C" {
                this.attr_int_c = attribute.unescape_and_decode_value(reader)?.parse()?;
            } else if attribute.key == b"F" {
                this.attr_int_f = attribute.unescape_and_decode_value(reader)?.parse()?;
            } else if attribute.key == b"G" {
                this.attr_int_g = attribute.unescape_and_decode_value(reader)?.parse()?;
            } else if attribute.key == b"H" {
                this.attr_tuple_h = parse_tuple(&attribute, reader)?;
            } else if attribute.key == b"I" {
                this.attr_float_i = attribute.unescape_and_decode_value(reader)?.parse()?;
            } else if attribute.key == b"J" {
                this.attr_int_j = attribute.unescape_and_decode_value(reader)?.parse()?;
            } else if attribute.key == b"K" {
                this.attr_int_k = attribute.unescape_and_decode_value(reader)?.parse()?;
            } else {
                anyhow::bail!(
                    "ControlNation found unknown attribute: {:?} {:?}",
                    String::from_utf8_lossy(attribute.key),
                    attribute.key
                );
            }
        }

        if !empty {
            loop {
                let event = reader.read_event(&mut buf)?;
                if let Event::End(bytes_end) = event {
                    let name = bytes_end.name();
                    if name == b"INFO" {
                        break;
                    }
                }
            }
        }

        Ok(this)
    }
}

fn parse_index(buf: &[u8]) -> anyhow::Result<usize> {
    let raw_str = String::from_utf8_lossy(buf);
    Ok(raw_str.parse()?)
}

fn parse_indices(buf: &[u8]) -> anyhow::Result<(usize, usize)> {
    let raw_str = String::from_utf8_lossy(buf);
    let index_values: Vec<usize> = raw_str
        .split('_')
        .map(|s| s.parse())
        .filter_map(Result::ok)
        .collect();

    let index_value_1: usize = *index_values
        .get(0)
        .with_context(|| format!("Failed to parse <Index>[0] in {:?}", &raw_str))?;

    let index_value_2: usize = *index_values
        .get(1)
        .with_context(|| format!("Failed to parse <Index>[1] in {:?}", &raw_str))?;

    Ok((index_value_1, index_value_2))
}

fn parse_tuple<T, R>(attribute: &Attribute, reader: &mut Reader<R>) -> anyhow::Result<(T, T)>
where
    T: FromStr + Clone,
    R: BufRead,
{
    let raw_str = attribute.unescape_and_decode_value(reader)?;
    let tuple_values: Vec<T> = raw_str
        .split_ascii_whitespace()
        .map(|s| s.parse())
        .filter_map(Result::ok)
        .collect();

    let tuple_value_1: T = tuple_values
        .get(0)
        .with_context(|| format!("Failed to parse Tuple[0] in {:?}", &raw_str))?
        .clone();

    let tuple_value_2: T = tuple_values
        .get(1)
        .with_context(|| format!("Failed to parse Tuple[1] in {:?}", &raw_str))?
        .clone();

    Ok((tuple_value_1, tuple_value_2))
}

fn parse_uv<R>(attribute: &Attribute, reader: &mut Reader<R>) -> anyhow::Result<Vec8f>
where
    R: BufRead,
{
    let raw_str = attribute.unescape_and_decode_value(reader)?;
    let uv_values: Vec<f64> = raw_str
        .split_ascii_whitespace()
        .map(|s| s.parse())
        .filter_map(Result::ok)
        .collect();

    Ok((
        *uv_values.get(0).context("Failed to parse <UV>[0]")?,
        *uv_values.get(1).context("Failed to parse <UV>[1]")?,
        *uv_values.get(2).context("Failed to parse <UV>[2]")?,
        *uv_values.get(3).context("Failed to parse <UV>[3]")?,
        *uv_values.get(4).context("Failed to parse <UV>[4]")?,
        *uv_values.get(5).context("Failed to parse <UV>[5]")?,
        *uv_values.get(6).context("Failed to parse <UV>[6]")?,
        *uv_values.get(7).context("Failed to parse <UV>[7]")?,
    ))
}
