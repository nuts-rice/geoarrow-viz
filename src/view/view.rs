use std::sync::Arc;
use winit::window::{Window, WindowId};
use crate::model::{GeoArrowFile, Bounds, GeoArrowResult};
use crate::error::GeoArrowError;
use web_sys::wasm_bindgen::JsCast;
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



pub struct MapView {
    pub position: (f64, f64),
    pub zoom: u8,
    pub bounds: Option<Bounds>, 
    pub id: i32,
    pub geoarrow_file: GeoArrowFile,
    pub style: MapStyle,
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

    //TODO: implement
    pub async fn render_to_canvas(&self, canvas_id: &str) -> GeoArrowResult<()>  {
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

        context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        context.set_fill_style_str(&self.style.polygon_fill.clone());
        context.set_stroke_style_str(&self.style.polygon_stroke.clone());
        context.set_line_width(self.style.line_width);
        


        tracing::info!("Rendering map {} to canvas {}", self.id, canvas_id);

        Ok(())

    }




}




