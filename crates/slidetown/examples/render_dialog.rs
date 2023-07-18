use anyhow::Context;
use image::*;
use imageproc::drawing::*;
use imageproc::rect;
use slidetown::parsers::nui::*;
use std::fs::File;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    let _ = args.next();
    let nui_path = args.next().context("Invalid nui path arg")?;
    let dialog_id: usize = args.next().context("Invalid dialog arg")?.parse()?;

    let nui_file = File::open(Path::new(&nui_path))?;
    let nui = Document::from_read(nui_file)?;

    let dialog = nui
        .dialog_list
        .dialogs
        .iter()
        .find(|d| d.id == dialog_id)
        .context("Failed to find specified dialog in nui")?;

    let mut img = RgbaImage::from_pixel(1280, 768, Pixel::from_channels(255u8, 255u8, 255u8, 0u8));

    draw_dialog(&mut img, dialog)?;

    img.save(Path::new("tmp.png"))?;

    Ok(())
}

fn draw_dialog(img: &mut RgbaImage, dialog: &Dialog) -> anyhow::Result<()> {
    let dimensions = img.dimensions();
    let red = Rgba([255u8, 0u8, 0u8, 255u8]);
    let green = Rgba([0u8, 255u8, 0u8, 255u8]);

    draw_hollow_rect_mut(
        img,
        rect::Rect::at(0, 0).of_size(dimensions.0, dimensions.1),
        green,
    );

    let alignment_mask = dialog.alignment_mask.unwrap_or(36) as usize;

    let (horiz_align, vert_align) = Alignment::from_usize_tuple(alignment_mask);
    dbg!(&horiz_align, &vert_align);

    let origin = (
        horiz_align.to_i32(dimensions.0 as i32),
        vert_align.to_i32(dimensions.1 as i32),
    );
    dbg!(&origin);

    for control in dialog.controls.iter() {
        draw_control(origin, control, img, red)?;
    }

    Ok(())
}

fn draw_control(
    origin: (i32, i32),
    control: &Control,
    img: &mut RgbaImage,
    debug_color: Rgba<u8>,
) -> anyhow::Result<()> {
    dbg!(&control.id, &control.text_id);

    if control.info.is_none() {
        println!("no control info");
        return Ok(());
    }

    let info = control.info.as_ref().unwrap();
    let attrs = &info.attributes;

    let pos = attrs.positions.get(&1);
    let size = attrs.sizes.get(&1);

    if pos.is_none() || size.is_none() {
        println!("no position/size found");
        return Ok(());
    }

    let mut pos = *pos.unwrap();
    if let Some(nation) = &control.nation {
        pos.0 += nation.attr_tuple_h.0;
        pos.1 += nation.attr_tuple_h.1;
    }
    let pos = (pos.0 as i32, pos.1 as i32);
    let size = size.unwrap();
    let size = (size.0 as u32, size.1 as u32);

    let pos = (pos.0 + origin.0, pos.1 + origin.1);
    dbg!(&pos, &size);

    let rect = rect::Rect::at(pos.0, pos.1).of_size(size.0, size.1);
    draw_hollow_rect_mut(img, rect, debug_color);

    Ok(())
}

#[derive(Debug)]
enum Alignment {
    None,
    Half,
    Full,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Full
    }
}

impl Alignment {
    pub fn from_usize(val: usize) -> Self {
        if val & 0b001 > 0 || val & 0b100 > 0 {
            Self::Full
        } else if val & 0b010 > 0 {
            Self::Half
        } else {
            Self::None
        }
    }

    pub fn from_usize_tuple(val: usize) -> (Self, Self) {
        let horizontal_raw = val & 0b111;
        let vertical_raw = (val >> 3) & 0b111;
        println!("horiz {:#b} vert {:#b}", horizontal_raw, vertical_raw);

        (
            Alignment::from_usize(horizontal_raw),
            Alignment::from_usize(vertical_raw),
        )
    }

    pub fn to_i32(&self, range: i32) -> i32 {
        match self {
            Alignment::None => 0,
            Alignment::Half => range / 2,
            Alignment::Full => range,
        }
    }
}
