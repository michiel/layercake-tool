use async_graphql::SimpleObject;

#[derive(SimpleObject)]
pub struct PaletteSwatch {
    pub name: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
    pub contrast_ratio: f64,
    pub passes_aa: bool,
}

#[derive(SimpleObject)]
pub struct Palette {
    pub name: String,
    pub description: String,
    pub swatches: Vec<PaletteSwatch>,
}

#[derive(SimpleObject)]
pub struct ContrastResult {
    pub ratio: f64,
    pub passes_aa: bool,
    pub passes_aa_large: bool,
    pub passes_aaa: bool,
}

impl From<layercake_core::palette::PaletteSwatch> for PaletteSwatch {
    fn from(s: layercake_core::palette::PaletteSwatch) -> Self {
        Self {
            name: s.name,
            background_color: s.background_color,
            text_color: s.text_color,
            border_color: s.border_color,
            contrast_ratio: s.contrast_ratio,
            passes_aa: s.passes_aa,
        }
    }
}

impl From<layercake_core::palette::Palette> for Palette {
    fn from(p: layercake_core::palette::Palette) -> Self {
        Self {
            name: p.name,
            description: p.description,
            swatches: p.swatches.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<layercake_core::palette::ContrastResult> for ContrastResult {
    fn from(c: layercake_core::palette::ContrastResult) -> Self {
        Self {
            ratio: c.ratio,
            passes_aa: c.passes_aa,
            passes_aa_large: c.passes_aa_large,
            passes_aaa: c.passes_aaa,
        }
    }
}
