use geoarrow_viz::{model::GeoArrowFile, view::view::MapView};
use wasm_bindgen::prelude::wasm_bindgen;
#[wasm_bindgen(start)]
pub fn main() {
    tracing_subscriber::fmt::init();
    let geoarrow_file = GeoArrowFile::new(
        "./sample_data.geojson".to_string(),
        0,
        "2025-01-01".to_string(),
    );
    let map_view = MapView::new(1, geoarrow_file, (10.0, 20.0), 15);

    map_view.render_to_canvas("canvas").unwrap();

    println!("Hello, world!");
}
