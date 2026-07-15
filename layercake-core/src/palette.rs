//! Layer colour palettes + WCAG contrast checking.
//!
//! Provides curated, accessibility-validated palettes agents can apply to
//! project layers, and a WCAG contrast calculator so a background/text pair can
//! be checked before it's used (a near-white-on-white palette "disappears" —
//! this catches that).

use serde::{Deserialize, Serialize};

/// One layer's colours.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteSwatch {
    pub name: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
    /// WCAG contrast ratio of text on background (1.0–21.0).
    pub contrast_ratio: f64,
    /// Passes WCAG AA for normal text (>= 4.5).
    pub passes_aa: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Palette {
    pub name: String,
    pub description: String,
    pub swatches: Vec<PaletteSwatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContrastResult {
    pub ratio: f64,
    pub passes_aa: bool,       // >= 4.5 (normal text)
    pub passes_aa_large: bool, // >= 3.0 (large text / UI)
    pub passes_aaa: bool,      // >= 7.0
}

/// Parse `#rrggbb` (or `rrggbb`) to (r,g,b) in 0..=255.
fn parse_hex(color: &str) -> Option<(u8, u8, u8)> {
    let h = color.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some((r, g, b))
}

fn channel_luminance(c: u8) -> f64 {
    let s = c as f64 / 255.0;
    if s <= 0.03928 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance of an sRGB colour.
fn relative_luminance((r, g, b): (u8, u8, u8)) -> f64 {
    0.2126 * channel_luminance(r) + 0.7152 * channel_luminance(g) + 0.0722 * channel_luminance(b)
}

/// WCAG contrast ratio between two colours (1.0–21.0). Returns None if either
/// colour can't be parsed.
pub fn contrast_ratio(a: &str, b: &str) -> Option<f64> {
    let la = relative_luminance(parse_hex(a)?);
    let lb = relative_luminance(parse_hex(b)?);
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    Some((hi + 0.05) / (lo + 0.05))
}

/// Full WCAG check for a background/text pair.
pub fn check_contrast(background: &str, text: &str) -> Option<ContrastResult> {
    let ratio = contrast_ratio(background, text)?;
    Some(ContrastResult {
        ratio: (ratio * 100.0).round() / 100.0,
        passes_aa: ratio >= 4.5,
        passes_aa_large: ratio >= 3.0,
        passes_aaa: ratio >= 7.0,
    })
}

fn swatch(name: &str, bg: &str, text: &str, border: &str) -> PaletteSwatch {
    let ratio = contrast_ratio(bg, text).unwrap_or(1.0);
    PaletteSwatch {
        name: name.into(),
        background_color: bg.into(),
        text_color: text.into(),
        border_color: border.into(),
        contrast_ratio: (ratio * 100.0).round() / 100.0,
        passes_aa: ratio >= 4.5,
    }
}

/// Curated palettes. All swatches are AA-compliant (text on background).
pub fn presets() -> Vec<Palette> {
    vec![
        Palette {
            name: "slate".into(),
            description: "Neutral slate tones, AA-compliant, distinct from a white UI".into(),
            swatches: vec![
                swatch("primary", "#1f2937", "#f9fafb", "#111827"),
                swatch("secondary", "#334155", "#f8fafc", "#1e293b"),
                swatch("accent", "#0e7490", "#ecfeff", "#155e75"),
                swatch("muted", "#475569", "#f1f5f9", "#334155"),
            ],
        },
        Palette {
            name: "warm".into(),
            description: "Warm earth tones, AA-compliant".into(),
            swatches: vec![
                swatch("primary", "#7c2d12", "#fff7ed", "#431407"),
                swatch("secondary", "#9a3412", "#fff7ed", "#7c2d12"),
                swatch("accent", "#a16207", "#fefce8", "#854d0e"),
                swatch("muted", "#78350f", "#fffbeb", "#451a03"),
            ],
        },
        Palette {
            name: "cool".into(),
            description: "Cool blues/greens, AA-compliant".into(),
            swatches: vec![
                swatch("primary", "#1e3a8a", "#eff6ff", "#1e40af"),
                swatch("secondary", "#065f46", "#ecfdf5", "#064e3b"),
                swatch("accent", "#5b21b6", "#f5f3ff", "#4c1d95"),
                swatch("muted", "#334155", "#f8fafc", "#1e293b"),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrast_ratio_extremes() {
        // Black on white is the maximum, ~21:1.
        let r = contrast_ratio("#000000", "#ffffff").unwrap();
        assert!((r - 21.0).abs() < 0.1, "{r}");
        // Same colour is 1:1.
        assert!((contrast_ratio("#777777", "#777777").unwrap() - 1.0).abs() < 0.01);
    }

    #[test]
    fn near_white_on_white_fails_aa() {
        // The kind of "disappears next to a white UI" pair the reviewer flagged.
        let c = check_contrast("#f7f7f8", "#ffffff").unwrap();
        assert!(!c.passes_aa, "near-white-on-white should fail AA: {:?}", c);
    }

    #[test]
    fn all_presets_pass_aa() {
        for p in presets() {
            for s in p.swatches {
                assert!(s.passes_aa, "{} / {} fails AA ({})", p.name, s.name, s.contrast_ratio);
            }
        }
    }
}
