use wasm_bindgen::prelude::*;
mod error;
use error::{GeoArrowError};
pub mod model;
use model::{Bounds, GeoArrowFile, GeoArrowResult  };
pub mod view;
use view::view::{MapView, MapStyle};


#[wasm_bindgen(start)]
fn start() {
    tracing_subscriber::fmt::init();



    println!("Hello, world!");
}
