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
//        let response = 
        Ok(())

    }

    pub fn file_path(&self) -> &str {
        &self.path
    }
}

