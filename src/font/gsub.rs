//! OpenType GSUB table builder supporting Lookup Type 4 (Ligature Substitution).

use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct LigatureEntry {
    /// First glyph ID in sequence
    pub first_glyph: u16,
    /// Subsequent glyph IDs in sequence (word_len - 1)
    pub components: Vec<u16>,
    /// Target composite glyph ID to display
    pub lig_glyph: u16,
}

pub struct GsubBuilder {
    /// Map from first_glyph to list of ligatures starting with that glyph
    ligatures_by_first: BTreeMap<u16, Vec<LigatureEntry>>,
}

impl Default for GsubBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GsubBuilder {
    pub fn new() -> Self {
        Self {
            ligatures_by_first: BTreeMap::new(),
        }
    }

    pub fn add_ligature(&mut self, first_glyph: u16, components: Vec<u16>, lig_glyph: u16) {
        self.ligatures_by_first
            .entry(first_glyph)
            .or_default()
            .push(LigatureEntry {
                first_glyph,
                components,
                lig_glyph,
            });
    }

    /// Serializes a complete, valid OpenType GSUB table binary.
    pub fn build(self) -> Vec<u8> {
        // Sort ligatures within each set by descending component count,
        // so longer matching prefixes take precedence (standard OpenType rule).
        let mut sorted_sets: BTreeMap<u16, Vec<LigatureEntry>> = BTreeMap::new();
        for (first, mut list) in self.ligatures_by_first {
            list.sort_by_key(|a| std::cmp::Reverse(a.components.len()));
            sorted_sets.insert(first, list);
        }

        let first_glyphs: Vec<u16> = sorted_sets.keys().copied().collect();
        let num_sets = first_glyphs.len();

        // --- Step 1: Build LigatureSubstFormat1 subtable ---
        // Header: substFormat (2) + coverageOffset (2) + ligSetCount (2) + ligSetOffsets (2 * num_sets)
        let lig_subst_header_len = 6 + 2 * num_sets;

        // CoverageFormat1: format (2) + glyphCount (2) + glyphArray (2 * num_sets)
        let coverage_offset = lig_subst_header_len;
        let coverage_len = 4 + 2 * num_sets;

        // Ligature sets follow coverage
        let mut lig_subst_body = Vec::new();
        let mut lig_set_offsets = Vec::with_capacity(num_sets);

        let mut current_set_offset = lig_subst_header_len + coverage_len;

        for &first in &first_glyphs {
            lig_set_offsets.push(current_set_offset as u16);
            let ligs = &sorted_sets[&first];

            // LigatureSet table:
            // ligatureCount (2) + ligatureOffsets (2 * count)
            let lig_set_header_len = 2 + 2 * ligs.len();
            let mut lig_offsets = Vec::with_capacity(ligs.len());
            let mut lig_tables_data = Vec::new();

            let mut curr_lig_offset = lig_set_header_len;
            for lig in ligs {
                lig_offsets.push(curr_lig_offset as u16);
                // Ligature table:
                // ligGlyph (2) + compCount (2) + componentGlyphIDs (2 * comp_len)
                let mut lig_data = Vec::new();
                lig_data.extend_from_slice(&lig.lig_glyph.to_be_bytes());
                let comp_count = (lig.components.len() + 1) as u16;
                lig_data.extend_from_slice(&comp_count.to_be_bytes());
                for &comp in &lig.components {
                    lig_data.extend_from_slice(&comp.to_be_bytes());
                }

                curr_lig_offset += lig_data.len();
                lig_tables_data.extend(lig_data);
            }

            let mut set_data = Vec::new();
            set_data.extend_from_slice(&(ligs.len() as u16).to_be_bytes());
            for off in lig_offsets {
                set_data.extend_from_slice(&off.to_be_bytes());
            }
            set_data.extend(lig_tables_data);

            current_set_offset += set_data.len();
            lig_subst_body.extend(set_data);
        }

        // Assemble LigatureSubstFormat1
        let mut lig_subst_table = Vec::new();
        lig_subst_table.extend_from_slice(&1u16.to_be_bytes()); // substFormat = 1
        lig_subst_table.extend_from_slice(&(coverage_offset as u16).to_be_bytes());
        lig_subst_table.extend_from_slice(&(num_sets as u16).to_be_bytes());
        for off in lig_set_offsets {
            lig_subst_table.extend_from_slice(&off.to_be_bytes());
        }

        // Append Coverage table
        lig_subst_table.extend_from_slice(&1u16.to_be_bytes()); // coverageFormat = 1
        lig_subst_table.extend_from_slice(&(num_sets as u16).to_be_bytes());
        for first in &first_glyphs {
            lig_subst_table.extend_from_slice(&first.to_be_bytes());
        }

        // Append LigatureSets
        lig_subst_table.extend(lig_subst_body);

        // --- Step 2: Build LookupList table ---
        // Lookup table:
        // lookupType (uint16 = 4)
        // lookupFlag (uint16 = 0)
        // subTableCount (uint16 = 1)
        // subtableOffsets[0] (Offset16 = 8, because header is 2+2+2+2 = 8 bytes)
        let mut lookup_table = Vec::new();
        lookup_table.extend_from_slice(&4u16.to_be_bytes()); // LookupType 4 = Ligature Substitution
        lookup_table.extend_from_slice(&0u16.to_be_bytes()); // lookupFlag = 0
        lookup_table.extend_from_slice(&1u16.to_be_bytes()); // subTableCount = 1
        lookup_table.extend_from_slice(&8u16.to_be_bytes()); // subtableOffset = 8 (points to lig_subst_table directly after header)
        lookup_table.extend(lig_subst_table);

        // LookupList: lookupCount (2 = 1) + lookupOffsets (2 = 4)
        let mut lookup_list = Vec::new();
        lookup_list.extend_from_slice(&1u16.to_be_bytes());
        lookup_list.extend_from_slice(&4u16.to_be_bytes());
        lookup_list.extend(lookup_table);

        // --- Step 3: Build FeatureList table ---
        // Feature table: featureParams (2 = 0) + lookupIndexCount (2 = 1) + lookupListIndices (2 = 0)
        let mut feature_table = Vec::new();
        feature_table.extend_from_slice(&0u16.to_be_bytes());
        feature_table.extend_from_slice(&1u16.to_be_bytes());
        feature_table.extend_from_slice(&0u16.to_be_bytes()); // Index 0 into LookupList

        // FeatureList:
        // featureCount (2 = 2)
        // FeatureRecord 0: "calt", offset
        // FeatureRecord 1: "liga", offset
        let feature_header_len = 2 + 2 * 6; // 14 bytes
        let calt_offset = feature_header_len as u16;
        let liga_offset = (feature_header_len + feature_table.len()) as u16;

        let mut feature_list = Vec::new();
        feature_list.extend_from_slice(&2u16.to_be_bytes()); // 2 features
        feature_list.extend_from_slice(b"calt");
        feature_list.extend_from_slice(&calt_offset.to_be_bytes());
        feature_list.extend_from_slice(b"liga");
        feature_list.extend_from_slice(&liga_offset.to_be_bytes());
        feature_list.extend(&feature_table);
        feature_list.extend(&feature_table);

        // --- Step 4: Build ScriptList table ---
        // LangSys table: lookupOrder (2 = 0) + reqFeatureIndex (2 = 0xFFFF) + featureIndexCount (2 = 2) + indices (0, 1)
        let mut lang_sys = Vec::new();
        lang_sys.extend_from_slice(&0u16.to_be_bytes());
        lang_sys.extend_from_slice(&0xFFFFu16.to_be_bytes());
        lang_sys.extend_from_slice(&2u16.to_be_bytes());
        lang_sys.extend_from_slice(&0u16.to_be_bytes()); // calt
        lang_sys.extend_from_slice(&1u16.to_be_bytes()); // liga

        // Script table: defaultLangSys (2 = 4) + langSysCount (2 = 0)
        let mut script_table = Vec::new();
        script_table.extend_from_slice(&4u16.to_be_bytes());
        script_table.extend_from_slice(&0u16.to_be_bytes());
        script_table.extend(lang_sys);

        // ScriptList:
        // scriptCount (2 = 2)
        // ScriptRecord 0: "DFLT", offset
        // ScriptRecord 1: "latn", offset
        let script_header_len = 2 + 2 * 6; // 14 bytes
        let dflt_offset = script_header_len as u16;
        let latn_offset = (script_header_len + script_table.len()) as u16;

        let mut script_list = Vec::new();
        script_list.extend_from_slice(&2u16.to_be_bytes());
        script_list.extend_from_slice(b"DFLT");
        script_list.extend_from_slice(&dflt_offset.to_be_bytes());
        script_list.extend_from_slice(b"latn");
        script_list.extend_from_slice(&latn_offset.to_be_bytes());
        script_list.extend(&script_table);
        script_list.extend(&script_table);

        // --- Step 5: Assemble GSUB Header ---
        // Header: majorVersion (2 = 1) + minorVersion (2 = 0) + scriptListOff (2) + featListOff (2) + lookupListOff (2)
        let header_len = 10;
        let script_offset = header_len as u16;
        let feat_offset = (header_len + script_list.len()) as u16;
        let lookup_offset = (header_len + script_list.len() + feature_list.len()) as u16;

        let mut gsub = Vec::new();
        gsub.extend_from_slice(&1u16.to_be_bytes()); // majorVersion = 1
        gsub.extend_from_slice(&0u16.to_be_bytes()); // minorVersion = 0
        gsub.extend_from_slice(&script_offset.to_be_bytes());
        gsub.extend_from_slice(&feat_offset.to_be_bytes());
        gsub.extend_from_slice(&lookup_offset.to_be_bytes());

        gsub.extend(script_list);
        gsub.extend(feature_list);
        gsub.extend(lookup_list);

        gsub
    }
}
