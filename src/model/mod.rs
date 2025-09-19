use geojson::FeatureCollection;   
use arrow::datatypes::Schema;
use wasm_bindgen::prelude::*;
use crate::error::GeoArrowError;
use std::fmt::Debug;
pub type GeoArrowResult<T> = Result<T, GeoArrowError>;

pub struct Bounds {
    pub min_x: f64, 
    pub min_y: f64, 
    pub max_x: f64,
    pub max_y: f64
}
impl Bounds {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Bounds {min_x, min_y, max_x, max_y}
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && y <= self.max_x && y >= self.min_y && y <= self.max_y
    }
}

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
        let content = if self.path.starts_with("http") || self.path.starts_with("https") {
            let resp = reqwest::get(&self.path).await.map_err(|e| GeoArrowError::Io(format!("Failed to fetch URL: {}", e)))?;
            resp.text().await.map_err(|e| GeoArrowError::Io(format!("Failed to read response: {}", e)))?
            } else {
            std::fs::read_to_string(&self.path).map_err(|e| GeoArrowError::Io(format!("Failed to read file {}: {}", self.path, e)))?
        };

        self.parse_content(&content)?;
        Ok(())
    }

    fn parse_content(&mut self, content: &str) -> GeoArrowResult<()> {
        // Determine file format based on extension or content
        if self.path.ends_with(".geojson") || self.path.ends_with(".json") {
            self.parse_geojson(content)?;
        } else if self.path.ends_with(".parquet") {
            return Err(GeoArrowError::Serialization("Parquet format not yet implemented".to_string()));
        } else {
            // Try to auto-detect format
            if content.trim_start().starts_with('{') || content.trim_start().starts_with('[') {
                self.parse_geojson(content)?;
            } else {
                return Err(GeoArrowError::Serialization("Unknown file format".to_string()));
            }
        }
        Ok(())
    }

    fn parse_geojson(&mut self, content: &str) -> GeoArrowResult<()> {
        let geojson: geojson::GeoJson = content.parse()
            .map_err(|e| GeoArrowError::Serialization(format!("Invalid GeoJSON: {}", e)))?;

        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => {
                self.feature_count = Some(fc.features.len());
                tracing::info!("Loaded {} features from GeoJSON", fc.features.len());
            }
            geojson::GeoJson::Feature(_) => {
                self.feature_count = Some(1);
                tracing::info!("Loaded single feature from GeoJSON");
            }
            geojson::GeoJson::Geometry(_) => {
                self.feature_count = Some(1);
                tracing::info!("Loaded single geometry from GeoJSON");
            }
        }

        // TODO: Convert to Arrow schema when geoarrow integration is ready
        self.schema = None;
        Ok(())
    }


    pub async fn get_features(&self) -> GeoArrowResult<FeatureCollection> {
        todo!()

    }

    pub fn file_path(&self) -> &str {
        &self.path
    }
}

