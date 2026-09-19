pub mod base;
pub mod builder;
pub mod gsub;

pub use base::{BaseFont, DEFAULT_BASE_FONT};
pub use builder::FontBuilder;
pub use gsub::GsubBuilder;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::tokenizer::ActiveLigature;
    use std::fs;
    use std::process::Command;

    #[test]
    fn test_build_font_with_ligatures() {
        let builder = FontBuilder::default();
        let ligs = vec![
            ActiveLigature {
                decoy: "engine".to_string(),
                original: "horse".to_string(),
            },
            ActiveLigature {
                decoy: "apple".to_string(),
                original: "potato".to_string(),
            },
        ];

        let font_bytes = builder.build_font(&ligs).expect("build font");
        assert!(!font_bytes.is_empty());
        fs::write("/tmp/test_font.ttf", &font_bytes).expect("write /tmp/test_font.ttf");

        let woff_bytes = builder.build_woff(&ligs).expect("build woff");
        assert!(!woff_bytes.is_empty());
        assert_eq!(&woff_bytes[0..4], b"wOFF", "Invalid WOFF signature");
        println!("TTF size: {} bytes, WOFF size: {} bytes (compression: {:.1}%)",
            font_bytes.len(),
            woff_bytes.len(),
            (1.0 - (woff_bytes.len() as f64 / font_bytes.len() as f64)) * 100.0
        );
        // WOFF should be substantially smaller than TTF
        assert!(woff_bytes.len() < font_bytes.len());
    }

    #[test]
    fn test_end_to_end_paragraph_1() {
        let text = "In the ancient kingdom, a brave engineer rode his horse through the forest toward the mountain castle.";
        let res = crate::guard_text(text);
        let builder = FontBuilder::default();
        let font_bytes = builder.build_font(&res.ligatures).expect("build font");
        fs::write("/tmp/p1_font.ttf", &font_bytes).expect("write font");

        // Verify with python HarfBuzz that every decoy word shapes into 1 glyph
        let script = r#"
import ctypes
hb = ctypes.CDLL('libharfbuzz.so.0')
hb.hb_blob_create_from_file.restype = ctypes.c_void_p
hb.hb_blob_create_from_file.argtypes = [ctypes.c_char_p]
hb.hb_face_create.restype = ctypes.c_void_p
hb.hb_face_create.argtypes = [ctypes.c_void_p, ctypes.c_uint]
hb.hb_font_create.restype = ctypes.c_void_p
hb.hb_font_create.argtypes = [ctypes.c_void_p]
hb.hb_buffer_create.restype = ctypes.c_void_p
hb.hb_buffer_add_utf8.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int, ctypes.c_uint, ctypes.c_int]
hb.hb_buffer_guess_segment_properties.argtypes = [ctypes.c_void_p]
hb.hb_shape.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_uint]
hb.hb_buffer_get_length.restype = ctypes.c_uint
hb.hb_buffer_get_length.argtypes = [ctypes.c_void_p]

blob = hb.hb_blob_create_from_file(b'/tmp/p1_font.ttf')
face = hb.hb_face_create(blob, 0)
font = hb.hb_font_create(face)

test_words = ['distant', 'sailor', 'flew', 'engine', 'valley', 'harbor']
for w in test_words:
    buf = hb.hb_buffer_create()
    wb = w.encode()
    hb.hb_buffer_add_utf8(buf, wb, len(wb), 0, len(wb))
    hb.hb_buffer_guess_segment_properties(buf)
    hb.hb_shape(font, buf, None, 0)
    count = hb.hb_buffer_get_length(buf)
    print(f'Word \"{w}\" -> glyph count: {count}')
    assert count == 1, f'Expected 1 ligature glyph for {w}, got {count}'
"#;
        let out = Command::new("python3")
            .arg("-c")
            .arg(script)
            .output()
            .expect("run python harfbuzz");
        println!("{}", String::from_utf8_lossy(&out.stdout));
        assert!(out.status.success(), "HarfBuzz shaping verification failed");
    }
}
