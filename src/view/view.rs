use crate::error::GeoArrowError;
use crate::model::{Bounds, GeoArrowFile, GeoArrowResult};
use std::sync::Arc;
use web_sys::wasm_bindgen::JsCast;
use winit::window::{Window, WindowId};
struct State {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    size: winit::dpi::PhysicalSize<u32>,
    surface_format: wgpu::TextureFormat,
}
#[derive(Clone)]
pub struct MapStyle {
    pub point_color: String,
    pub line_color: String,
    pub polygon_fill: String,
    pub polygon_stroke: String,
    pub point_radius: f64,
    pub line_width: f64,
}
impl Default for MapStyle {
    fn default() -> Self {
        MapStyle {
            point_color: "#FF0000".to_string(),
            line_color: "#0000FF".to_string(),
            polygon_fill: "rgba(0, 255, 0, 0.3)".to_string(),
            polygon_stroke: "#00FF00".to_string(),
            point_radius: 3.0,
            line_width: 2.0,
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
pub struct MapView {
    position: (f64, f64),
    zoom: u8,
    bounds: Option<Bounds>,
    id: i32,
    geoarrow_file: GeoArrowFile,
    style: MapStyle,
}

impl Default for MapView {
    fn default() -> Self {
        MapView {
            position: (0.0, 0.0),
            zoom: 1,
            bounds: None,
            id: 0,
            geoarrow_file: GeoArrowFile::new(
                "./data/Utah.geojson".to_string(),
                0,
                "2023-01-01".to_string(),
            ),
            style: MapStyle::default(),
        }
    }
}

impl MapView {
    pub fn new(id: i32, geoarrow_file: GeoArrowFile, position: (f64, f64), zoom: u8) -> Self {
        MapView {
            id,
            zoom,
            geoarrow_file,
            bounds: None,
            position,
            style: MapStyle::default(),
        }
    }

    pub fn set_position(&mut self, position: (f64, f64)) {
        self.position = position;
    }

    pub fn get_position(&self) -> (f64, f64) {
        self.position
    }

    pub fn set_zoom(&mut self, zoom: u8) {
        self.zoom = zoom;
    }

    pub fn get_zoom(&self) -> u8 {
        self.zoom
    }

    pub fn render_to_canvas(&self, canvas_id: &str) -> GeoArrowResult<()> {
        let document = web_sys::window()
            .ok_or_else(|| GeoArrowError::Wasm("No window".to_string()))?
            .document()
            .ok_or_else(|| GeoArrowError::Wasm("No document".to_string()))?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| GeoArrowError::Wasm(format!("Canvas {} not found", canvas_id)))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| GeoArrowError::Wasm("Element is not a canvas".to_string()))?;

        let context = canvas
            .get_context("2d")
            .map_err(|_| GeoArrowError::Wasm("Could not get 2d context".to_string()))?
            .ok_or_else(|| GeoArrowError::Wasm("No 2d context".to_string()))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| GeoArrowError::Wasm("Context is not 2d".to_string()))?;

        // Clear canvas
        context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        // Set up basic styling
        context.set_fill_style_str(&self.style.polygon_fill);
        context.set_stroke_style_str(&self.style.polygon_stroke);
        context.set_line_width(self.style.line_width);

        // Draw simple crosshairs to show the map center
        let center_x = canvas.width() as f64 / 2.0;
        let center_y = canvas.height() as f64 / 2.0;

        context.begin_path();
        context.move_to(center_x - 10.0, center_y);
        context.line_to(center_x + 10.0, center_y);
        context.move_to(center_x, center_y - 10.0);
        context.line_to(center_x, center_y + 10.0);
        context.stroke();

        // TODO: Implement actual geospatial data rendering
        // This would involve:
        // 1. Loading data from self.geoarrow_file
        // 2. Transforming coordinates based on self.position and self.zoom
        // 3. Rendering features (points, lines, polygons) as tiles

        tracing::info!(
            "Rendered map {} to canvas {} at position {:?}, zoom {}",
            self.id,
            canvas_id,
            self.position,
            self.zoom
        );

        Ok(())
    }
}

#[wasm_bindgen::prelude::wasm_bindgen]
impl MapView {
    #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
    pub fn new_wasm() -> MapView {
        MapView::default()
    }

    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn render_to_canvas_wasm(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
        self.render_to_canvas(canvas_id)
            .map_err(|e| wasm_bindgen::JsValue::from_str(&format!("Rendering error: {}", e)))
    }

    #[wasm_bindgen::prelude::wasm_bindgen(getter)]
    pub fn zoom(&self) -> u8 {
        self.zoom
    }

    #[wasm_bindgen::prelude::wasm_bindgen(setter)]
    pub fn set_zoom_wasm(&mut self, zoom: u8) {
        self.zoom = zoom;
    }

    #[wasm_bindgen::prelude::wasm_bindgen(getter)]
    pub fn position_x(&self) -> f64 {
        self.position.0
    }

    #[wasm_bindgen::prelude::wasm_bindgen(getter)]
    pub fn position_y(&self) -> f64 {
        self.position.1
    }

    #[wasm_bindgen::prelude::wasm_bindgen]
    pub fn set_position_wasm(&mut self, x: f64, y: f64) {
        self.position = (x, y);
    }
}

mod test {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_map_view_creation() {
        let map_view = MapView::default();
        assert_eq!(map_view.get_zoom(), 1);
    }
}
