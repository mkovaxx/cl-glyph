use wasm_bindgen::{JsError, JsValue, prelude::wasm_bindgen};

use crate::Glyph;

/// Turn a JSON-encoded `Glyph` into an SVG.
#[wasm_bindgen]
pub fn glyph_to_svg(js: JsValue) -> Result<String, JsError> {
    let glyph: Glyph = serde_wasm_bindgen::from_value(js)?;
    let svg = glyph.to_svg();
    let svg_text = svg.to_string();
    Ok(svg_text)
}
