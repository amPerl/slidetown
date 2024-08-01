use std::{cell::RefCell, rc::Rc};

use binrw::BinWrite;

pub type EntryOffsets = Rc<RefCell<Vec<u64>>>;

#[binrw::writer(writer, endian)]
pub(crate) fn record_entry_offset(
    &value: &u32,
    offsets: Option<EntryOffsets>,
) -> binrw::BinResult<()> {
    if let Some(entry_offsets) = offsets {
        let pos = writer.stream_position()?;
        entry_offsets.borrow_mut().push(pos);
    }
    value.write_options(writer, endian, ())
}
