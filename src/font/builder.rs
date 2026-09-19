//! Font synthesizer that generates TrueType composite glyphs and GSUB ligature tables.

use std::collections::BTreeMap;
use std::io::Write;
use super::base::{BaseFont, DEFAULT_BASE_FONT};
use super::gsub::GsubBuilder;
use crate::dictionary::tokenizer::ActiveLigature;

pub struct FontBuilder<'a> {
    base: BaseFont<'a>,
}

impl Default for FontBuilder<'static> {
    fn default() -> Self {
        Self::from_bytes(DEFAULT_BASE_FONT).expect("default base font is valid")
    }
}

impl<'a> FontBuilder<'a> {
    /// Creates a builder using the provided TrueType font byte slice.
    pub fn new(base_font_data: &'a [u8]) -> Result<Self, String> {
        Self::from_bytes(base_font_data)
    }

    /// Creates a builder from borrowed TrueType font bytes (e.g. `include_bytes!`).
    pub fn from_bytes(base_font_data: &'a [u8]) -> Result<Self, String> {
        let base = BaseFont::from_bytes(base_font_data)?;
        Ok(Self { base })
    }

    /// Creates a builder from an owned font byte vector.
    pub fn from_vec(base_font_data: Vec<u8>) -> Result<FontBuilder<'static>, String> {
        let base = BaseFont::from_vec(base_font_data)?;
        Ok(FontBuilder { base })
    }

    /// Loads a TrueType font from a file path.
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<FontBuilder<'static>, String> {
        let p = path.as_ref();
        let bytes = std::fs::read(p).map_err(|e| format!("Failed to read font file {}: {}", p.display(), e))?;
        Self::from_vec(bytes)
    }

    /// Creates a builder by checking the `TEXTGUARD_FONT_PATH` environment variable.
    /// Falls back to the default embedded font if not set or unreadable.
    pub fn from_env() -> FontBuilder<'static> {
        Self::from_env_with_lookup(|key| std::env::var(key).ok())
    }

    /// Creates a builder using a custom environment lookup closure.
    pub fn from_env_with_lookup<F>(lookup: F) -> FontBuilder<'static>
    where
        F: Fn(&str) -> Option<String>,
    {
        if let Some(path) = lookup("TEXTGUARD_FONT_PATH") {
            match Self::from_file(&path) {
                Ok(b) => return b,
                Err(e) => {
                    eprintln!(
                        "Warning: failed to load TEXTGUARD_FONT_PATH ({}): {}. Falling back to default font.",
                        path, e
                    );
                }
            }
        }
        FontBuilder::default()
    }

    /// Synthesizes all required font tables (cmap, glyf, loca, hmtx, GSUB, etc.)
    /// with composite glyphs and GSUB ligatures for the active substitutions.
    pub fn generate_tables(&self, ligatures: &[ActiveLigature]) -> Result<BTreeMap<[u8; 4], Vec<u8>>, String> {
        let base_data = self.base.data.as_ref();

        // Collect existing tables we want to preserve
        let cmap_rec = self.base.tables.get(b"cmap").ok_or("cmap table missing")?;
        let head_rec = self.base.tables.get(b"head").ok_or("head table missing")?;
        let hhea_rec = self.base.tables.get(b"hhea").ok_or("hhea table missing")?;
        let loca_rec = self.base.tables.get(b"loca").ok_or("loca table missing")?;
        let glyf_rec = self.base.tables.get(b"glyf").ok_or("glyf table missing")?;
        let maxp_rec = self.base.tables.get(b"maxp").ok_or("maxp table missing")?;
        let name_rec = self.base.tables.get(b"name").ok_or("name table missing")?;
        let os2_rec = self.base.tables.get(b"OS/2").ok_or("OS/2 table missing")?;

        let old_num_glyphs = self.base.num_glyphs;

        // Clone base glyf, loca, and hmtx data
        let mut new_glyf = base_data[glyf_rec.offset..glyf_rec.offset + glyf_rec.length].to_vec();
        let mut new_loca = base_data[loca_rec.offset..loca_rec.offset + loca_rec.length].to_vec();

        // Convert base hmtx to a flat list of longHorMetric (advanceWidth, lsb) for all old glyphs
        let mut hmtx_entries: Vec<(u16, i16)> = Vec::with_capacity(old_num_glyphs as usize + ligatures.len());
        for gid in 0..old_num_glyphs {
            hmtx_entries.push(self.base.get_metrics(gid));
        }

        let mut gsub = GsubBuilder::new();
        let mut next_gid = old_num_glyphs;

        for lig in ligatures {
            // Resolve glyph IDs for the input sequence (decoy word, e.g. "engine")
            let mut input_gids = Vec::new();
            for ch in lig.decoy.chars() {
                if let Some(gid) = self.base.get_glyph_id(ch) {
                    input_gids.push(gid);
                } else {
                    return Err(format!("Char '{}' in decoy '{}' not found in base font", ch, lig.decoy));
                }
            }

            // Resolve glyph IDs for the target word to render (original word, e.g. "horse")
            let mut original_gids = Vec::new();
            for ch in lig.original.chars() {
                if let Some(gid) = self.base.get_glyph_id(ch) {
                    original_gids.push(gid);
                } else {
                    return Err(format!("Char '{}' in original '{}' not found in base font", ch, lig.original));
                }
            }

            if input_gids.is_empty() || original_gids.is_empty() {
                continue;
            }

            let new_gid = next_gid;
            next_gid += 1;

            // 1. Calculate bounding box and composite glyph components for original word
            let mut x_cursor: i32 = 0;
            let mut comp_bytes = Vec::new();
            let mut overall_x_min = i16::MAX;
            let mut overall_y_min = i16::MAX;
            let mut overall_x_max = i16::MIN;
            let mut overall_y_max = i16::MIN;

            let count = original_gids.len();
            for (idx, &orig_gid) in original_gids.iter().enumerate() {
                let (aw, _lsb) = self.base.get_metrics(orig_gid);
                let (gx_min, gy_min, gx_max, gy_max) = self.base.get_glyph_bbox(orig_gid);

                let is_last = idx == count - 1;
                // ARG_1_AND_2_ARE_WORDS (0x0001) | ARGS_ARE_XY_VALUES (0x0002)
                let mut flags: u16 = 0x0001 | 0x0002;
                if !is_last {
                    // MORE_COMPONENTS (0x0020)
                    flags |= 0x0020;
                }

                comp_bytes.extend_from_slice(&flags.to_be_bytes());
                comp_bytes.extend_from_slice(&orig_gid.to_be_bytes());
                comp_bytes.extend_from_slice(&(x_cursor as i16).to_be_bytes()); // e
                comp_bytes.extend_from_slice(&0i16.to_be_bytes());               // f

                if gx_min != 0 || gx_max != 0 || gy_min != 0 || gy_max != 0 {
                    overall_x_min = overall_x_min.min((gx_min as i32 + x_cursor) as i16);
                    overall_x_max = overall_x_max.max((gx_max as i32 + x_cursor) as i16);
                    overall_y_min = overall_y_min.min(gy_min);
                    overall_y_max = overall_y_max.max(gy_max);
                }

                x_cursor += aw as i32;
            }

            if overall_x_min == i16::MAX {
                overall_x_min = 0;
                overall_x_max = x_cursor as i16;
                overall_y_min = 0;
                overall_y_max = 1000;
            }

            // Build new glyph header
            let mut glyph_data = Vec::new();
            glyph_data.extend_from_slice(&(-1i16).to_be_bytes()); // numberOfContours = -1 for composite
            glyph_data.extend_from_slice(&overall_x_min.to_be_bytes());
            glyph_data.extend_from_slice(&overall_y_min.to_be_bytes());
            glyph_data.extend_from_slice(&overall_x_max.to_be_bytes());
            glyph_data.extend_from_slice(&overall_y_max.to_be_bytes());
            glyph_data.extend_from_slice(&comp_bytes);

            // Pad glyph to 2-byte boundary
            if glyph_data.len() % 2 != 0 {
                glyph_data.push(0);
            }

            // Append glyph to new_glyf and record offset in new_loca
            let glyph_start_offset = new_glyf.len() as u32;
            new_glyf.extend_from_slice(&glyph_data);

            // Append offset to loca table (Format 1: 32-bit offsets)
            new_loca.extend_from_slice(&glyph_start_offset.to_be_bytes());

            // Append metrics: advance width = total width of original word, lsb = overall_x_min
            hmtx_entries.push((x_cursor as u16, overall_x_min));

            // Register ligature substitution in GSUB
            let first_gid = input_gids[0];
            let component_gids = input_gids[1..].to_vec();
            gsub.add_ligature(first_gid, component_gids, new_gid);
        }

        // Final entry in loca points to the end of glyf data
        let final_offset = new_glyf.len() as u32;
        new_loca.extend_from_slice(&final_offset.to_be_bytes());

        // Construct new hmtx table
        let mut new_hmtx = Vec::with_capacity(hmtx_entries.len() * 4);
        for (aw, lsb) in &hmtx_entries {
            new_hmtx.extend_from_slice(&aw.to_be_bytes());
            new_hmtx.extend_from_slice(&lsb.to_be_bytes());
        }

        // Update maxp table: numGlyphs
        let mut new_maxp = base_data[maxp_rec.offset..maxp_rec.offset + maxp_rec.length].to_vec();
        new_maxp[4..6].copy_from_slice(&next_gid.to_be_bytes());

        // Update hhea table: numberOfHMetrics = next_gid
        let mut new_hhea = base_data[hhea_rec.offset..hhea_rec.offset + hhea_rec.length].to_vec();
        new_hhea[34..36].copy_from_slice(&next_gid.to_be_bytes());

        // Update head table: clear checkSumAdjustment for recalculation
        let mut new_head = base_data[head_rec.offset..head_rec.offset + head_rec.length].to_vec();
        new_head[8..12].copy_from_slice(&[0, 0, 0, 0]);
        // Force indexToLocFormat = 1 (32-bit offsets)
        new_head[50..52].copy_from_slice(&1i16.to_be_bytes());

        // Generate GSUB table binary
        let gsub_table = gsub.build();

        // Assemble all tables
        let mut tables = BTreeMap::new();
        tables.insert(*b"cmap", base_data[cmap_rec.offset..cmap_rec.offset + cmap_rec.length].to_vec());
        tables.insert(*b"glyf", new_glyf);
        tables.insert(*b"head", new_head);
        tables.insert(*b"hhea", new_hhea);
        tables.insert(*b"hmtx", new_hmtx);
        tables.insert(*b"loca", new_loca);
        tables.insert(*b"maxp", new_maxp);
        tables.insert(*b"name", base_data[name_rec.offset..name_rec.offset + name_rec.length].to_vec());
        tables.insert(*b"OS/2", base_data[os2_rec.offset..os2_rec.offset + os2_rec.length].to_vec());
        tables.insert(*b"GSUB", gsub_table);

        Ok(tables)
    }

    /// Assembles an uncompressed TrueType SFNT container with the synthesized tables.
    pub fn build_font(&self, ligatures: &[ActiveLigature]) -> Result<Vec<u8>, String> {
        let tables = self.generate_tables(ligatures)?;

        // Calculate table offsets and sizes
        let num_tables = tables.len() as u16;
        let mut search_range: u16 = 1;
        let mut entry_selector: u16 = 0;
        while search_range * 2 <= num_tables {
            search_range *= 2;
            entry_selector += 1;
        }
        search_range *= 16;
        let range_shift = num_tables * 16 - search_range;

        let header_size = 12 + num_tables as usize * 16;
        let mut current_offset = header_size;

        // Compute offsets and pad tables
        let mut table_dir = Vec::new();
        let mut table_payloads = Vec::new();

        let table_keys: Vec<[u8; 4]> = tables.keys().cloned().collect();
        for tag in table_keys {
            let data = tables.get(&tag).unwrap();
            let length = data.len();
            let checksum = compute_table_checksum(data);

            table_dir.push((tag, checksum, current_offset as u32, length as u32));

            let mut padded = data.clone();
            while padded.len() % 4 != 0 {
                padded.push(0);
            }
            current_offset += padded.len();
            table_payloads.push(padded);
        }

        // Assemble binary
        let mut font = Vec::with_capacity(current_offset);
        // SFNT Header
        font.extend_from_slice(&0x00010000u32.to_be_bytes()); // TrueType scaler type
        font.extend_from_slice(&num_tables.to_be_bytes());
        font.extend_from_slice(&search_range.to_be_bytes());
        font.extend_from_slice(&entry_selector.to_be_bytes());
        font.extend_from_slice(&range_shift.to_be_bytes());

        // Table Directory
        for (tag, checksum, offset, length) in &table_dir {
            font.extend_from_slice(tag);
            font.extend_from_slice(&checksum.to_be_bytes());
            font.extend_from_slice(&offset.to_be_bytes());
            font.extend_from_slice(&length.to_be_bytes());
        }

        // Table Data
        for payload in table_payloads {
            font.extend_from_slice(&payload);
        }

        // Compute checkSumAdjustment for head table
        let entire_checksum = compute_font_checksum(&font);
        let checksum_adjustment = 0xB1B0AFBA_u32.wrapping_sub(entire_checksum);

        let head_offset = for_tag_offset(&tables, *b"head", header_size);
        font[head_offset + 8..head_offset + 12].copy_from_slice(&checksum_adjustment.to_be_bytes());

        // Also update checksum in table directory for head
        for i in 0..num_tables as usize {
            let entry_off = 12 + i * 16;
            if &font[entry_off..entry_off + 4] == b"head" {
                let head_len = u32::from_be_bytes([
                    font[entry_off + 12], font[entry_off + 13], font[entry_off + 14], font[entry_off + 15]
                ]) as usize;
                let head_data = &font[head_offset..head_offset + head_len];
                let new_head_sum = compute_table_checksum(head_data);
                font[entry_off + 4..entry_off + 8].copy_from_slice(&new_head_sum.to_be_bytes());
                break;
            }
        }

        Ok(font)
    }

    /// Assembles a compressed WOFF 1.0 container from the synthesized tables.
    pub fn build_woff(&self, ligatures: &[ActiveLigature]) -> Result<Vec<u8>, String> {
        let mut tables = self.generate_tables(ligatures)?;

        // First compute checkSumAdjustment by generating SFNT layout
        let sfnt_bytes = self.build_font(ligatures)?;
        let head_checksum_adj = &sfnt_bytes[for_tag_offset(&tables, *b"head", 12 + tables.len() * 16) + 8..for_tag_offset(&tables, *b"head", 12 + tables.len() * 16) + 12];
        if let Some(head) = tables.get_mut(b"head") {
            head[8..12].copy_from_slice(head_checksum_adj);
        }

        let num_tables = tables.len() as u16;
        let total_sfnt_size = sfnt_bytes.len();

        let header_size = 44 + (num_tables as usize * 20);
        let mut current_offset = header_size;

        let mut woff_dir_entries = Vec::new();
        let mut woff_payloads = Vec::new();

        let tables_to_write: Vec<([u8; 4], Vec<u8>)> = tables.into_iter().collect();

        for (tag, data) in &tables_to_write {
            let orig_len = data.len() as u32;
            let orig_checksum = compute_table_checksum(data);

            // Compress table data with zlib
            let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(data).map_err(|e| e.to_string())?;
            let compressed = encoder.finish().map_err(|e| e.to_string())?;

            let (comp_len, table_data) = if compressed.len() < data.len() {
                (compressed.len() as u32, compressed)
            } else {
                (orig_len, data.clone())
            };

            woff_dir_entries.push((*tag, current_offset as u32, comp_len, orig_len, orig_checksum));

            let mut padded = table_data;
            while padded.len() % 4 != 0 {
                padded.push(0);
            }
            current_offset += padded.len();
            woff_payloads.push(padded);
        }

        let total_woff_size = current_offset as u32;

        let mut woff = Vec::with_capacity(total_woff_size as usize);
        // WOFF 1.0 Header (44 bytes)
        woff.extend_from_slice(b"wOFF");
        woff.extend_from_slice(&0x00010000u32.to_be_bytes()); // flavor
        woff.extend_from_slice(&total_woff_size.to_be_bytes()); // length
        woff.extend_from_slice(&num_tables.to_be_bytes()); // numTables
        woff.extend_from_slice(&0u16.to_be_bytes()); // reserved
        woff.extend_from_slice(&(total_sfnt_size as u32).to_be_bytes()); // totalSfntSize
        woff.extend_from_slice(&1u16.to_be_bytes()); // majorVersion
        woff.extend_from_slice(&0u16.to_be_bytes()); // minorVersion
        woff.extend_from_slice(&0u32.to_be_bytes()); // metaOffset
        woff.extend_from_slice(&0u32.to_be_bytes()); // metaLength
        woff.extend_from_slice(&0u32.to_be_bytes()); // metaOrigLength
        woff.extend_from_slice(&0u32.to_be_bytes()); // privOffset
        woff.extend_from_slice(&0u32.to_be_bytes()); // privLength

        // Table Directory (20 bytes per table)
        for (tag, offset, comp_len, orig_len, orig_checksum) in woff_dir_entries {
            woff.extend_from_slice(&tag);
            woff.extend_from_slice(&offset.to_be_bytes());
            woff.extend_from_slice(&comp_len.to_be_bytes());
            woff.extend_from_slice(&orig_len.to_be_bytes());
            woff.extend_from_slice(&orig_checksum.to_be_bytes());
        }

        // Payloads
        for payload in woff_payloads {
            woff.extend(payload);
        }

        Ok(woff)
    }
}

fn for_tag_offset(tables: &BTreeMap<[u8; 4], Vec<u8>>, target_tag: [u8; 4], base_offset: usize) -> usize {
    let mut off = base_offset;
    for (tag, data) in tables {
        if *tag == target_tag {
            return off;
        }
        let mut len = data.len();
        while len % 4 != 0 {
            len += 1;
        }
        off += len;
    }
    base_offset
}

fn compute_table_checksum(table_data: &[u8]) -> u32 {
    let mut sum = 0u32;
    let (chunks, remainder) = table_data.as_chunks::<4>();
    for chunk in chunks {
        let val = u32::from_be_bytes(*chunk);
        sum = sum.wrapping_add(val);
    }
    if !remainder.is_empty() {
        let mut pad = [0u8; 4];
        pad[..remainder.len()].copy_from_slice(remainder);
        let val = u32::from_be_bytes(pad);
        sum = sum.wrapping_add(val);
    }
    sum
}

fn compute_font_checksum(font_data: &[u8]) -> u32 {
    let mut sum = 0u32;
    let (chunks, remainder) = font_data.as_chunks::<4>();
    for chunk in chunks {
        let val = u32::from_be_bytes(*chunk);
        sum = sum.wrapping_add(val);
    }
    if !remainder.is_empty() {
        let mut pad = [0u8; 4];
        pad[..remainder.len()].copy_from_slice(remainder);
        let val = u32::from_be_bytes(pad);
        sum = sum.wrapping_add(val);
    }
    sum
}
