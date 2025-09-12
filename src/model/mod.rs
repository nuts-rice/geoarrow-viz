use geojson::FeatureCollection;   
use arrow::datatypes::Schema;
use wasm_bindgen::prelude::*;
use crate::error::GeoArrowError;
use std::fmt::Debug;
pub type GeoArrowResult<T> = Result<T, GeoArrowError>;

pub type Bounds = (f64, f64, f64, f64);

pub struct GeoArrowFile {
    pub path: String,
    pub size: i64,
    pub created_at: String,
    pub schema: Option<Schema>,
    pub feature_count: Option<usize>,
}

impl Debug for GeoArrowFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GeoArrowFile {{ path: {}, size: {}, created_at: {}, schema: {:?}, feature_count: {:?} }}",
            self.path, self.size, self.created_at, self.schema, self.feature_count
        )
    }
}


impl GeoArrowFile {
    pub fn new(path: String, size: i64, created_at: String) -> Self {
        GeoArrowFile {
            path,
            size,
            created_at,
            schema: None,
            feature_count: None
        }
    }

    pub async fn open(&mut self) -> GeoArrowResult<()> {
        self.load_from_url().await?;
        Ok(())
    }

    async fn load_from_url(&mut self) -> GeoArrowResult<()> {
        tracing::info!("Loading geoarrow file from URL: {}", self.path);
        let response = 
        Ok(())

    }

    pub fn file_path(&self) -> &str {
        &self.path
    }
}

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

}



