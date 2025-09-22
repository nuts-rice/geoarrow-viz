use wasm_bindgen::prelude::*;
mod error;
mod model;
mod view;
use view::view::MapView;

#[wasm_bindgen(start)]
pub fn start() {
    tracing_subscriber::fmt::init();

    tracing::info!("Starting GeoArrow visualization engine");
    let map_view = MapView::default();
    map_view.render_to_canvas("canvas").unwrap();
}
