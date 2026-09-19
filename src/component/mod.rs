//! Topcoat `<Shield>` component and SSR template integration.

use base64::Engine;
use topcoat::view::{View, component, view};

use crate::dictionary::tokenizer::{ActiveLigature, ShieldResult, TextGuardEngine};
use crate::font::builder::FontBuilder;

/// Topcoat `<Shield>` component.
///
/// Wraps prose in an anti-scraping consent layer. Emits:
/// 1. A scoped `@font-face` stylesheet with dynamically synthesized OpenType GSUB ligatures.
/// 2. The visual decoy text (`aria-hidden="true"`).
/// 3. A screen-reader-accessible container (`sr-only`) with the genuine author prose.
///
/// # Accessibility
///
/// When `a11y` is enabled (default `true`), assistive technology receives clean,
/// uncorrupted text with no OCR or deciphering needed.
///
/// # Dynamic Involutions & Custom Fonts
///
/// - Passing `seed: Some(u64)` dynamically permutes cohyponym word pools into
///   fresh bijective involutions per page, session, or author salt.
/// - Passing `base_font: Some(&'a [u8])` utilizes custom TrueType font bytes (e.g. `include_bytes!("Inter.ttf")`).
#[component]
pub async fn Shield<'a>(
    /// The authored prose to guard.
    text: &'a str,
    /// Optional dynamic seed or salt for dynamic involution permutations.
    #[default(None)]
    seed: Option<u64>,
    /// Optional custom TrueType base font bytes (e.g. `include_bytes!("Inter.ttf")`).
    #[default(None)]
    base_font: Option<&'a [u8]>,
    /// Accessibility mode: provides clean original text for assistive screen readers.
    #[default(true)]
    a11y: bool,
    /// Additional CSS classes to attach to the container.
    #[default("")]
    class: &'a str,
) -> topcoat::Result<impl View> {
    let engine = match seed {
        Some(s) => TextGuardEngine::with_seed(s),
        None => TextGuardEngine::new(),
    };
    let result = engine.transform(text);

    let font_builder = match base_font {
        Some(bytes) => FontBuilder::from_bytes(bytes).unwrap_or_else(|e| {
            eprintln!(
                "Warning: failed to parse custom base_font: {}. Falling back to default font.",
                e
            );
            FontBuilder::default()
        }),
        None => FontBuilder::from_env(),
    };

    let (font_bytes, format_str, mime_str) = match font_builder.build_woff(&result.ligatures) {
        Ok(b) if !b.is_empty() => (b, "woff", "font/woff"),
        _ => (
            font_builder
                .build_font(&result.ligatures)
                .unwrap_or_default(),
            "truetype",
            "font/truetype",
        ),
    };
    let font_b64 = base64::engine::general_purpose::STANDARD.encode(&font_bytes);

    let id = format!("{:08x}", fnv1a_hash(&result.decoy_text));
    let font_family = format!("tg-{}", id);
    let wrapper_class = if class.is_empty() {
        format!("tg-sec-{}", id)
    } else {
        format!("tg-sec-{} {}", id, class)
    };

    let style = format!(
        "@font-face {{ font-family: '{}'; src: url('data:{};charset=utf-8;base64,{}') format('{}'); font-display: block; }} \
         .{} {{ font-family: '{}', sans-serif !important; font-variant-ligatures: common-ligatures contextual !important; font-feature-settings: 'liga' on !important; -webkit-font-feature-settings: 'liga' on !important; }} \
         .sr-only {{ position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border-width: 0; }}",
        font_family, mime_str, font_b64, format_str, wrapper_class, font_family
    );

    Ok(view! {
        <span class=(wrapper_class)>
            <style>(style)</style>
            // Visual text: Scrapers reading DOM see decoy; browser font ligatures render genuine author text
            <span aria-hidden="true">(result.decoy_text)</span>
            // Accessible text: Screen readers and assistive tools read original authored text
            if a11y {
                <span class="sr-only" aria-hidden="false">(result.original_text)</span>
            }
        </span>
    })
}

/// Computes a deterministic 64-bit FNV-1a hash of a string slice.
pub fn fnv1a_hash(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Convenience function to guard text in code without full component rendering.
pub fn guard_text(text: &str) -> ShieldResult {
    TextGuardEngine::new().transform(text)
}

/// Helper to transform text with a dynamic integer seed.
pub fn guard_text_with_seed(text: &str, seed: u64) -> ShieldResult {
    TextGuardEngine::with_seed(seed).transform(text)
}

/// Helper to transform text with a string salt (e.g. article slug or tenant ID).
pub fn guard_text_with_salt(text: &str, salt: &str) -> ShieldResult {
    TextGuardEngine::with_salt(salt).transform(text)
}

/// Helper to generate OpenType TrueType font bytes for custom ligatures.
pub fn build_shield_font(ligatures: &[ActiveLigature]) -> Result<Vec<u8>, String> {
    FontBuilder::default().build_font(ligatures)
}

/// Helper to generate compressed WOFF 1.0 font bytes for custom ligatures.
pub fn build_shield_woff(ligatures: &[ActiveLigature]) -> Result<Vec<u8>, String> {
    FontBuilder::default().build_woff(ligatures)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::base::DEFAULT_BASE_FONT;
    use topcoat::context::Cx;
    use topcoat::view::ViewExt;

    #[tokio::test]
    async fn test_shield_render() {
        let cx = Cx::default();
        let __cx = &cx;
        let input = "The knight rode his horse into battle.";
        let v = view! {
            Shield(text: input, a11y: true)
        };
        let rendered = v.single().await.expect("single").render(&cx);

        assert!(rendered.contains("@font-face"));
        assert!(
            rendered.contains("data:font/woff;charset=utf-8;base64,")
                || rendered.contains("data:font/truetype;charset=utf-8;base64,")
        );
        assert!(rendered.contains("engine"));
        assert!(rendered.contains("aria-hidden=\"true\""));
        assert!(rendered.contains("horse"));
        assert!(rendered.contains("class=\"sr-only\""));
    }

    #[tokio::test]
    async fn test_shield_render_with_seed() {
        let cx = Cx::default();
        let __cx = &cx;
        let input = "The doctor joined the brave pilot on the island.";
        let v = view! {
            Shield(text: input, seed: Some(7777), a11y: true)
        };
        let rendered = v.single().await.expect("single").render(&cx);

        assert!(rendered.contains("@font-face"));
        assert!(rendered.contains("aria-hidden=\"true\""));
        assert!(rendered.contains("class=\"sr-only\""));
    }

    #[tokio::test]
    async fn test_shield_render_with_custom_font() {
        let cx = Cx::default();
        let __cx = &cx;
        let input = "The doctor joined the brave pilot on the island.";
        let v = view! {
            Shield(text: input, base_font: Some(DEFAULT_BASE_FONT), a11y: true)
        };
        let rendered = v.single().await.expect("single").render(&cx);

        assert!(rendered.contains("@font-face"));
        assert!(rendered.contains("aria-hidden=\"true\""));
        assert!(rendered.contains("class=\"sr-only\""));
    }
}
