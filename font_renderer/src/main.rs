use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::family_name::FamilyName;
use font_kit::hinting::HintingOptions;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{Vector2F, Vector2I};
pub fn main() {
    let font = SystemSource::new()
        .select_best_match(&[FamilyName::SansSerif], &Properties::new())
        .unwrap()
        .load()
        .unwrap();
    let glyph_id = font.glyph_for_char('A').unwrap();

    let bounds = font.typographic_bounds(glyph_id).ok().unwrap();
    let mut canvas = Canvas::new(Vector2I::splat(32), Format::A8);
    println!("{} {}", bounds.width(), bounds.height());
    let raster_bounds = font
        .raster_bounds(
            glyph_id,
            12.0,
            Transform2F::from_translation(Vector2F::new(0.0, 32.0)),
            HintingOptions::None,
            RasterizationOptions::GrayscaleAa,
        )
        .unwrap();
    println!("{} {}", raster_bounds.width(), raster_bounds.height());
}
