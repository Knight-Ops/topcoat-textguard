//! Parsing base TrueType / OpenType font tables and glyph metrics.

use std::borrow::Cow;
use std::collections::HashMap;

/// Embed the clean TrueType base font included with textguard.
pub static DEFAULT_BASE_FONT: &[u8] = include_bytes!("../../assets/base.ttf");

#[derive(Debug, Clone, Copy)]
pub struct TableRecord {
    pub tag: [u8; 4],
    pub checksum: u32,
    pub offset: usize,
    pub length: usize,
}

pub struct BaseFont<'a> {
    pub data: Cow<'a, [u8]>,
    pub tables: HashMap<[u8; 4], TableRecord>,
    pub num_glyphs: u16,
    pub index_to_loc_format: i16,
    pub num_h_metrics: u16,
    pub cmap_subtable_offset: usize,
}

impl<'a> BaseFont<'a> {
    /// Parses font tables from a borrowed byte slice.
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, String> {
        Self::parse(Cow::Borrowed(data))
    }

    /// Parses font tables from an owned byte vector.
    pub fn from_vec(data: Vec<u8>) -> Result<BaseFont<'static>, String> {
        BaseFont::parse(Cow::Owned(data))
    }

    /// Parses a TrueType font binary into tables and metrics.
    pub fn parse(data: Cow<'a, [u8]>) -> Result<Self, String> {
        let slice = data.as_ref();
        if slice.len() < 12 {
            return Err("Font data too small for SFNT header".into());
        }

        let num_tables = u16::from_be_bytes([slice[4], slice[5]]) as usize;
        if slice.len() < 12 + num_tables * 16 {
            return Err("Font truncated in table directory".into());
        }

        let mut tables = HashMap::new();
        for i in 0..num_tables {
            let off = 12 + i * 16;
            let mut tag = [0u8; 4];
            tag.copy_from_slice(&slice[off..off + 4]);
            let checksum = u32::from_be_bytes([
                slice[off + 4],
                slice[off + 5],
                slice[off + 6],
                slice[off + 7],
            ]);
            let offset = u32::from_be_bytes([
                slice[off + 8],
                slice[off + 9],
                slice[off + 10],
                slice[off + 11],
            ]) as usize;
            let length = u32::from_be_bytes([
                slice[off + 12],
                slice[off + 13],
                slice[off + 14],
                slice[off + 15],
            ]) as usize;
            tables.insert(
                tag,
                TableRecord {
                    tag,
                    checksum,
                    offset,
                    length,
                },
            );
        }

        // Parse maxp for numGlyphs
        let maxp_rec = tables.get(b"maxp").ok_or("Missing maxp table")?;
        let num_glyphs =
            u16::from_be_bytes([slice[maxp_rec.offset + 4], slice[maxp_rec.offset + 5]]);

        // Parse head for indexToLocFormat
        let head_rec = tables.get(b"head").ok_or("Missing head table")?;
        let index_to_loc_format =
            i16::from_be_bytes([slice[head_rec.offset + 50], slice[head_rec.offset + 51]]);

        // Parse hhea for numberOfHMetrics
        let hhea_rec = tables.get(b"hhea").ok_or("Missing hhea table")?;
        let num_h_metrics =
            u16::from_be_bytes([slice[hhea_rec.offset + 34], slice[hhea_rec.offset + 35]]);

        // Parse cmap for Unicode BMP format 4 subtable
        let cmap_rec = tables.get(b"cmap").ok_or("Missing cmap table")?;
        let num_cmap_subtables =
            u16::from_be_bytes([slice[cmap_rec.offset + 2], slice[cmap_rec.offset + 3]]) as usize;
        let mut chosen_subtable = None;

        for i in 0..num_cmap_subtables {
            let sub_rec = cmap_rec.offset + 4 + i * 8;
            let platform_id = u16::from_be_bytes([slice[sub_rec], slice[sub_rec + 1]]);
            let encoding_id = u16::from_be_bytes([slice[sub_rec + 2], slice[sub_rec + 3]]);
            let sub_offset = cmap_rec.offset
                + u32::from_be_bytes([
                    slice[sub_rec + 4],
                    slice[sub_rec + 5],
                    slice[sub_rec + 6],
                    slice[sub_rec + 7],
                ]) as usize;

            let format = u16::from_be_bytes([slice[sub_offset], slice[sub_offset + 1]]);
            if format == 4
                && ((platform_id == 0 && (encoding_id == 3 || encoding_id == 4))
                    || (platform_id == 3 && encoding_id == 1))
            {
                chosen_subtable = Some(sub_offset);
                break;
            }
        }

        let cmap_subtable_offset =
            chosen_subtable.ok_or("No supported format 4 cmap subtable found")?;

        Ok(Self {
            data,
            tables,
            num_glyphs,
            index_to_loc_format,
            num_h_metrics,
            cmap_subtable_offset,
        })
    }

    /// Looks up glyph ID for a Unicode character using format 4 subtable.
    pub fn get_glyph_id(&self, c: char) -> Option<u16> {
        let code = (c as u32) as u16;
        let data = self.data.as_ref();
        let sub = self.cmap_subtable_offset;

        let seg_count = u16::from_be_bytes([data[sub + 6], data[sub + 7]]) as usize / 2;
        let end_codes_offset = sub + 14;
        let start_codes_offset = end_codes_offset + seg_count * 2 + 2;
        let id_delta_offset = start_codes_offset + seg_count * 2;
        let id_range_offset = id_delta_offset + seg_count * 2;

        for i in 0..seg_count {
            let end_code = u16::from_be_bytes([
                data[end_codes_offset + i * 2],
                data[end_codes_offset + i * 2 + 1],
            ]);
            if end_code >= code {
                let start_code = u16::from_be_bytes([
                    data[start_codes_offset + i * 2],
                    data[start_codes_offset + i * 2 + 1],
                ]);
                if start_code <= code {
                    let id_range = u16::from_be_bytes([
                        data[id_range_offset + i * 2],
                        data[id_range_offset + i * 2 + 1],
                    ]);
                    let id_delta = i16::from_be_bytes([
                        data[id_delta_offset + i * 2],
                        data[id_delta_offset + i * 2 + 1],
                    ]);
                    if id_range == 0 {
                        let gid = ((code as i32 + id_delta as i32) & 0xffff) as u16;
                        return if gid > 0 { Some(gid) } else { None };
                    } else {
                        let glyph_index_addr = id_range_offset
                            + i * 2
                            + id_range as usize
                            + (code - start_code) as usize * 2;
                        if glyph_index_addr + 1 < data.len() {
                            let raw_gid = u16::from_be_bytes([
                                data[glyph_index_addr],
                                data[glyph_index_addr + 1],
                            ]);
                            if raw_gid != 0 {
                                let gid = ((raw_gid as i32 + id_delta as i32) & 0xffff) as u16;
                                return if gid > 0 { Some(gid) } else { None };
                            }
                        }
                    }
                }
                break;
            }
        }
        None
    }

    /// Retrieves advance width and left side bearing for a glyph ID from `hmtx`.
    pub fn get_metrics(&self, gid: u16) -> (u16, i16) {
        let hmtx_rec = match self.tables.get(b"hmtx") {
            Some(r) => r,
            None => return (1000, 0),
        };
        let gid_usize = gid as usize;
        let num_h = self.num_h_metrics as usize;
        let off = hmtx_rec.offset;
        let data = self.data.as_ref();

        if gid_usize < num_h {
            let entry_off = off + gid_usize * 4;
            if entry_off + 4 <= data.len() {
                let aw = u16::from_be_bytes([data[entry_off], data[entry_off + 1]]);
                let lsb = i16::from_be_bytes([data[entry_off + 2], data[entry_off + 3]]);
                return (aw, lsb);
            }
        } else {
            let last_aw_off = off + (num_h.saturating_sub(1)) * 4;
            let aw = if last_aw_off + 2 <= data.len() {
                u16::from_be_bytes([data[last_aw_off], data[last_aw_off + 1]])
            } else {
                1000
            };
            let lsb_off = off + num_h * 4 + (gid_usize - num_h) * 2;
            let lsb = if lsb_off + 2 <= data.len() {
                i16::from_be_bytes([data[lsb_off], data[lsb_off + 1]])
            } else {
                0
            };
            return (aw, lsb);
        }
        (1000, 0)
    }

    /// Looks up glyph bounding box from `glyf` and `loca`.
    pub fn get_glyph_bbox(&self, gid: u16) -> (i16, i16, i16, i16) {
        let loca_rec = match self.tables.get(b"loca") {
            Some(r) => r,
            None => return (0, 0, 1000, 1000),
        };
        let glyf_rec = match self.tables.get(b"glyf") {
            Some(r) => r,
            None => return (0, 0, 1000, 1000),
        };

        let gid_usize = gid as usize;
        let data = self.data.as_ref();

        let (glyph_offset, next_offset) = if self.index_to_loc_format == 1 {
            let o1 = loca_rec.offset + gid_usize * 4;
            let o2 = loca_rec.offset + (gid_usize + 1) * 4;
            if o2 + 4 > data.len() {
                return (0, 0, 1000, 1000);
            }
            let off1 =
                u32::from_be_bytes([data[o1], data[o1 + 1], data[o1 + 2], data[o1 + 3]]) as usize;
            let off2 =
                u32::from_be_bytes([data[o2], data[o2 + 1], data[o2 + 2], data[o2 + 3]]) as usize;
            (off1, off2)
        } else {
            let o1 = loca_rec.offset + gid_usize * 2;
            let o2 = loca_rec.offset + (gid_usize + 1) * 2;
            if o2 + 2 > data.len() {
                return (0, 0, 1000, 1000);
            }
            let off1 = u16::from_be_bytes([data[o1], data[o1 + 1]]) as usize * 2;
            let off2 = u16::from_be_bytes([data[o2], data[o2 + 1]]) as usize * 2;
            (off1, off2)
        };

        if glyph_offset >= next_offset {
            // Empty glyph (e.g. space)
            return (0, 0, 0, 0);
        }

        let abs_glyph = glyf_rec.offset + glyph_offset;
        if abs_glyph + 10 <= data.len() {
            let x_min = i16::from_be_bytes([data[abs_glyph + 2], data[abs_glyph + 3]]);
            let y_min = i16::from_be_bytes([data[abs_glyph + 4], data[abs_glyph + 5]]);
            let x_max = i16::from_be_bytes([data[abs_glyph + 6], data[abs_glyph + 7]]);
            let y_max = i16::from_be_bytes([data[abs_glyph + 8], data[abs_glyph + 9]]);
            (x_min, y_min, x_max, y_max)
        } else {
            (0, 0, 1000, 1000)
        }
    }
}
