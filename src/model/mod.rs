use crate::error::GeoArrowError;
use arrow::datatypes::Schema;
use dashmap::DashMap;
use geojson::{Feature, FeatureCollection, Geometry, Position, Value as GeoValue};
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};
pub type GeoArrowResult<T> = Result<T, GeoArrowError>;

#[derive(Clone, Debug, PartialEq)]
pub struct GeoBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PixelBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TileBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

// Legacy alias for backward compatibility
pub type Bounds = GeoBounds;

impl GeoBounds {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        GeoBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    pub fn is_empty(&self) -> bool {
        self.min_x >= self.max_x || self.min_y >= self.max_y
    }

    pub fn intersects(&self, other: &GeoBounds) -> bool {
        !(self.max_x <= other.min_x
            || self.min_x >= other.max_x
            || self.max_y <= other.min_y
            || self.min_y >= other.max_y)
    }
}

impl PixelBounds {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        PixelBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }
}

impl TileBounds {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        TileBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn from_tile_coords(x: u32, y: u32, z: u8) -> Self {
        let tile_size = 1.0 / (1u32 << z) as f64;
        let min_x = x as f64 * tile_size;
        let min_y = y as f64 * tile_size;
        TileBounds::new(min_x, min_y, min_x + tile_size, min_y + tile_size)
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
        write!(
            f,
            "GeoArrowFile {{ path: {}, size: {}, created_at: {}, schema: {:?}, feature_count: {:?} }}",
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
            feature_count: None,
        }
    }
    pub async fn open(&mut self) -> GeoArrowResult<()> {
        self.load_from_url().await?;
        Ok(())
    }

    async fn load_from_url(&mut self) -> GeoArrowResult<()> {
        tracing::info!("Loading geoarrow file from URL: {}", self.path);
        let content = if self.path.starts_with("http") || self.path.starts_with("https") {
            let resp = reqwest::get(&self.path)
                .await
                .map_err(|e| GeoArrowError::Io(format!("Failed to fetch URL: {}", e)))?;
            resp.text()
                .await
                .map_err(|e| GeoArrowError::Io(format!("Failed to read response: {}", e)))?
        } else {
            std::fs::read_to_string(&self.path).map_err(|e| {
                GeoArrowError::Io(format!("Failed to read file {}: {}", self.path, e))
            })?
        };

        self.parse_content(&content)?;
        Ok(())
    }

    fn parse_content(&mut self, content: &str) -> GeoArrowResult<()> {
        // Determine file format based on extension or content
        if self.path.ends_with(".geojson") || self.path.ends_with(".json") {
            self.parse_geojson(content)?;
        } else if self.path.ends_with(".parquet") {
            return Err(GeoArrowError::Serialization(
                "Parquet format not yet implemented".to_string(),
            ));
        } else {
            // Try to auto-detect format
            if content.trim_start().starts_with('{') || content.trim_start().starts_with('[') {
                self.parse_geojson(content)?;
            } else {
                return Err(GeoArrowError::Serialization(
                    "Unknown file format".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn parse_geojson(&mut self, content: &str) -> GeoArrowResult<()> {
        let geojson: geojson::GeoJson = content
            .parse()
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
        // Load and parse the content first if not already done
        if self.feature_count.is_none() {
            return Err(GeoArrowError::Io(
                "File not loaded. Call open() first.".to_string(),
            ));
        }

        let content = if self.path.starts_with("http") || self.path.starts_with("https") {
            let resp = reqwest::get(&self.path)
                .await
                .map_err(|e| GeoArrowError::Io(format!("Failed to fetch URL: {}", e)))?;
            resp.text()
                .await
                .map_err(|e| GeoArrowError::Io(format!("Failed to read response: {}", e)))?
        } else {
            std::fs::read_to_string(&self.path).map_err(|e| {
                GeoArrowError::Io(format!("Failed to read file {}: {}", self.path, e))
            })?
        };

        let geojson: geojson::GeoJson = content
            .parse()
            .map_err(|e| GeoArrowError::Serialization(format!("Invalid GeoJSON: {}", e)))?;

        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => Ok(fc),
            geojson::GeoJson::Feature(f) => {
                let mut fc = FeatureCollection {
                    bbox: None,
                    features: vec![f],
                    foreign_members: None,
                };
                Ok(fc)
            }
            geojson::GeoJson::Geometry(g) => {
                let feature = Feature {
                    bbox: None,
                    geometry: Some(g),
                    id: None,
                    properties: None,
                    foreign_members: None,
                };
                let fc = FeatureCollection {
                    bbox: None,
                    features: vec![feature],
                    foreign_members: None,
                };
                Ok(fc)
            }
        }
    }

    pub fn file_path(&self) -> &str {
        &self.path
    }
}

// Core data models for tile-based visualization

// Unique identifiers
pub type FeatureId = String;
pub type LayerId = String;
pub type Timestamp = u64;

// Geographic point (latitude, longitude)
#[derive(Clone, Debug, PartialEq)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

impl GeoPoint {
    pub fn new(lat: f64, lng: f64) -> Self {
        GeoPoint { lat, lng }
    }

    pub fn is_valid(&self) -> bool {
        self.lat >= -90.0 && self.lat <= 90.0 && self.lng >= -180.0 && self.lng <= 180.0
    }
}

// Pixel size
#[derive(Clone, Debug, PartialEq)]
pub struct PixelSize {
    pub width: u32,
    pub height: u32,
}

impl PixelSize {
    pub fn new(width: u32, height: u32) -> Self {
        PixelSize { width, height }
    }
}

// Tile status enumeration
#[derive(Clone, Debug, PartialEq)]
pub enum TileStatus {
    NotLoaded,
    Loading,
    Loaded,
    Error(String),
}

// Tile structure representing a rendered rectangular segment
#[derive(Clone, Debug)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub z: u8,
    pub bounds: TileBounds,
    pub features: Vec<GeoFeature>,
    pub status: TileStatus,
    pub last_accessed: Timestamp,
}

impl Tile {
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        if z > 20 {
            panic!("Zoom level must be 0-20");
        }

        let bounds = TileBounds::from_tile_coords(x, y, z);
        let last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Tile {
            x,
            y,
            z,
            bounds,
            features: Vec::new(),
            status: TileStatus::NotLoaded,
            last_accessed,
        }
    }

    pub fn is_valid_for_zoom(&self, z: u8) -> bool {
        let max_coord = 1u32 << z;
        self.x < max_coord && self.y < max_coord && self.z == z
    }

    pub fn update_access_time(&mut self) {
        self.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    pub fn add_feature(&mut self, feature: GeoFeature) -> GeoArrowResult<()> {
        if !feature.bounds.intersects(&GeoBounds {
            min_x: self.bounds.min_x,
            min_y: self.bounds.min_y,
            max_x: self.bounds.max_x,
            max_y: self.bounds.max_y,
        }) {
            return Err(GeoArrowError::Serialization(
                "Feature does not intersect tile bounds".to_string(),
            ));
        }
        self.features.push(feature);
        Ok(())
    }
}

// Feature structure with geometry and properties
#[derive(Clone, Debug)]
pub struct GeoFeature {
    pub id: FeatureId,
    pub geometry: FeatureGeometry,
    pub properties: DashMap<String, serde_json::Value>,
    pub bounds: GeoBounds,
}

impl GeoFeature {
    pub fn new(
        id: FeatureId,
        geometry: FeatureGeometry,
        properties: DashMap<String, serde_json::Value>,
    ) -> Self {
        let bounds = geometry.calculate_bounds();
        GeoFeature {
            id,
            geometry,
            properties,
            bounds,
        }
    }

    pub fn from_geojson_feature(feature: &Feature) -> GeoArrowResult<Self> {
        let id = feature
            .id
            .as_ref()
            .map(|id| match id {
                geojson::feature::Id::String(s) => s.clone(),
                geojson::feature::Id::Number(n) => n.to_string(),
            })
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let geometry = if let Some(geom) = &feature.geometry {
            FeatureGeometry::from_geojson_geometry(geom)?
        } else {
            return Err(GeoArrowError::Serialization(
                "Feature has no geometry".to_string(),
            ));
        };

        if let Some(props) = &feature.properties {
            let properties: DashMap<String, serde_json::Value> = DashMap::new();
        }
        todo!()
    }
}

// Geometry types for features
#[derive(Clone, Debug)]
pub enum FeatureGeometry {
    Point(GeoPoint),
    LineString(Vec<GeoPoint>),
    Polygon(Vec<Vec<GeoPoint>>), // Exterior ring + holes
    MultiPoint(Vec<GeoPoint>),
    MultiLineString(Vec<Vec<GeoPoint>>),
    MultiPolygon(Vec<Vec<Vec<GeoPoint>>>),
}

impl FeatureGeometry {
    pub fn from_geojson_geometry(geometry: &Geometry) -> GeoArrowResult<Self> {
        match &geometry.value {
            GeoValue::Point(coords) => {
                let point = GeoPoint::new(coords[1], coords[0]); // lat, lng
                if !point.is_valid() {
                    return Err(GeoArrowError::Serialization(
                        "Invalid point coordinates".to_string(),
                    ));
                }
                Ok(FeatureGeometry::Point(point))
            }
            GeoValue::LineString(coords) => {
                let points: Result<Vec<_>, _> = coords
                    .iter()
                    .map(|pos| {
                        let point = GeoPoint::new(pos[1], pos[0]);
                        if point.is_valid() {
                            Ok(point)
                        } else {
                            Err(GeoArrowError::Serialization(
                                "Invalid line coordinates".to_string(),
                            ))
                        }
                    })
                    .collect();
                Ok(FeatureGeometry::LineString(points?))
            }
            GeoValue::Polygon(rings) => {
                let polygon_rings: Result<Vec<_>, _> = rings
                    .iter()
                    .map(|ring| {
                        ring.iter()
                            .map(|pos| {
                                let point = GeoPoint::new(pos[1], pos[0]);
                                if point.is_valid() {
                                    Ok(point)
                                } else {
                                    Err(GeoArrowError::Serialization(
                                        "Invalid polygon coordinates".to_string(),
                                    ))
                                }
                            })
                            .collect()
                    })
                    .collect();
                Ok(FeatureGeometry::Polygon(polygon_rings?))
            }
            GeoValue::MultiPoint(coords) => {
                let points: Result<Vec<_>, _> = coords
                    .iter()
                    .map(|pos| {
                        let point = GeoPoint::new(pos[1], pos[0]);
                        if point.is_valid() {
                            Ok(point)
                        } else {
                            Err(GeoArrowError::Serialization(
                                "Invalid multipoint coordinates".to_string(),
                            ))
                        }
                    })
                    .collect();
                Ok(FeatureGeometry::MultiPoint(points?))
            }
            GeoValue::MultiLineString(lines) => {
                let line_strings: Result<Vec<_>, _> = lines
                    .iter()
                    .map(|line| {
                        line.iter()
                            .map(|pos| {
                                let point = GeoPoint::new(pos[1], pos[0]);
                                if point.is_valid() {
                                    Ok(point)
                                } else {
                                    Err(GeoArrowError::Serialization(
                                        "Invalid multilinestring coordinates".to_string(),
                                    ))
                                }
                            })
                            .collect()
                    })
                    .collect();
                Ok(FeatureGeometry::MultiLineString(line_strings?))
            }
            GeoValue::MultiPolygon(polygons) => {
                let multi_polygon: Result<Vec<_>, _> = polygons
                    .iter()
                    .map(|rings| {
                        rings
                            .iter()
                            .map(|ring| {
                                ring.iter()
                                    .map(|pos| {
                                        let point = GeoPoint::new(pos[1], pos[0]);
                                        if point.is_valid() {
                                            Ok(point)
                                        } else {
                                            Err(GeoArrowError::Serialization(
                                                "Invalid multipolygon coordinates".to_string(),
                                            ))
                                        }
                                    })
                                    .collect()
                            })
                            .collect()
                    })
                    .collect();
                Ok(FeatureGeometry::MultiPolygon(multi_polygon?))
            }
            GeoValue::GeometryCollection(_) => Err(GeoArrowError::Serialization(
                "GeometryCollection not yet supported".to_string(),
            )),
        }
    }

    pub fn calculate_bounds(&self) -> GeoBounds {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        let update_bounds = |point: &GeoPoint,
                             min_x: &mut f64,
                             min_y: &mut f64,
                             max_x: &mut f64,
                             max_y: &mut f64| {
            *min_x = min_x.min(point.lng);
            *min_y = min_y.min(point.lat);
            *max_x = max_x.max(point.lng);
            *max_y = max_y.max(point.lat);
        };

        match self {
            FeatureGeometry::Point(point) => {
                update_bounds(point, &mut min_x, &mut min_y, &mut max_x, &mut max_y);
            }
            FeatureGeometry::LineString(points) | FeatureGeometry::MultiPoint(points) => {
                for point in points {
                    update_bounds(point, &mut min_x, &mut min_y, &mut max_x, &mut max_y);
                }
            }
            FeatureGeometry::Polygon(rings) => {
                for ring in rings {
                    for point in ring {
                        update_bounds(point, &mut min_x, &mut min_y, &mut max_x, &mut max_y);
                    }
                }
            }
            FeatureGeometry::MultiLineString(lines) => {
                for line in lines {
                    for point in line {
                        update_bounds(point, &mut min_x, &mut min_y, &mut max_x, &mut max_y);
                    }
                }
            }
            FeatureGeometry::MultiPolygon(polygons) => {
                for polygon in polygons {
                    for ring in polygon {
                        for point in ring {
                            update_bounds(point, &mut min_x, &mut min_y, &mut max_x, &mut max_y);
                        }
                    }
                }
            }
        }

        GeoBounds::new(min_x, min_y, max_x, max_y)
    }

    pub fn is_valid(&self) -> bool {
        match self {
            FeatureGeometry::Point(point) => point.is_valid(),
            FeatureGeometry::LineString(points) | FeatureGeometry::MultiPoint(points) => {
                !points.is_empty() && points.iter().all(|p| p.is_valid())
            }
            FeatureGeometry::Polygon(rings) => {
                !rings.is_empty()
                    && rings.iter().all(|ring| {
                        ring.len() >= 4
                            && ring.iter().all(|p| p.is_valid())
                            && ring.first() == ring.last() // Closed ring
                    })
            }
            FeatureGeometry::MultiLineString(lines) => {
                !lines.is_empty()
                    && lines
                        .iter()
                        .all(|line| !line.is_empty() && line.iter().all(|p| p.is_valid()))
            }
            FeatureGeometry::MultiPolygon(polygons) => {
                !polygons.is_empty()
                    && polygons.iter().all(|rings| {
                        !rings.is_empty()
                            && rings.iter().all(|ring| {
                                ring.len() >= 4
                                    && ring.iter().all(|p| p.is_valid())
                                    && ring.first() == ring.last()
                            })
                    })
            }
        }
    }
}

// Data source enumeration
#[derive(Clone, Debug)]
pub enum DataSource {
    Local(std::path::PathBuf),
    Http(String),
    Memory(Vec<u8>),
}

impl DataSource {
    pub fn from_path(path: &str) -> Self {
        if path.starts_with("http://") || path.starts_with("https://") {
            DataSource::Http(path.to_string())
        } else {
            DataSource::Local(std::path::PathBuf::from(path))
        }
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, DataSource::Http(_))
    }

    pub fn as_string(&self) -> String {
        match self {
            DataSource::Local(path) => path.to_string_lossy().to_string(),
            DataSource::Http(url) => url.clone(),
            DataSource::Memory(_) => "<memory>".to_string(),
        }
    }
}

// Layer styling configuration
#[derive(Clone, Debug)]
pub struct LayerStyle {
    pub point_style: PointStyle,
    pub line_style: LineStyle,
    pub polygon_style: PolygonStyle,
}

#[derive(Clone, Debug)]
pub struct PointStyle {
    pub color: String,
    pub radius: f64,
    pub opacity: f32,
}

#[derive(Clone, Debug)]
pub struct LineStyle {
    pub color: String,
    pub width: f64,
    pub opacity: f32,
    pub dash_pattern: Option<Vec<f64>>,
}

#[derive(Clone, Debug)]
pub struct PolygonStyle {
    pub fill_color: String,
    pub stroke_color: String,
    pub stroke_width: f64,
    pub fill_opacity: f32,
    pub stroke_opacity: f32,
}

impl Default for LayerStyle {
    fn default() -> Self {
        LayerStyle {
            point_style: PointStyle {
                color: "#FF0000".to_string(),
                radius: 3.0,
                opacity: 1.0,
            },
            line_style: LineStyle {
                color: "#0000FF".to_string(),
                width: 2.0,
                opacity: 1.0,
                dash_pattern: None,
            },
            polygon_style: PolygonStyle {
                fill_color: "rgba(0, 255, 0, 0.3)".to_string(),
                stroke_color: "#00FF00".to_string(),
                stroke_width: 1.0,
                fill_opacity: 0.3,
                stroke_opacity: 1.0,
            },
        }
    }
}

// Layer management
#[derive(Clone, Debug)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub data_source: DataSource,
    pub style: LayerStyle,
    pub visible: bool,
    pub z_index: i32,
    pub opacity: f32,
    pub min_zoom: u8,
    pub max_zoom: u8,
}

impl Layer {
    pub fn new(id: LayerId, name: String, data_source: DataSource) -> Self {
        Layer {
            id,
            name,
            data_source,
            style: LayerStyle::default(),
            visible: true,
            z_index: 0,
            opacity: 1.0,
            min_zoom: 0,
            max_zoom: 20,
        }
    }

    pub fn is_valid(&self) -> GeoArrowResult<()> {
        if self.id.is_empty() {
            return Err(GeoArrowError::Serialization(
                "Layer ID cannot be empty".to_string(),
            ));
        }
        if self.name.is_empty() {
            return Err(GeoArrowError::Serialization(
                "Layer name cannot be empty".to_string(),
            ));
        }
        if self.opacity < 0.0 || self.opacity > 1.0 {
            return Err(GeoArrowError::Serialization(
                "Layer opacity must be between 0.0 and 1.0".to_string(),
            ));
        }
        if self.min_zoom > self.max_zoom {
            return Err(GeoArrowError::Serialization(
                "min_zoom cannot be greater than max_zoom".to_string(),
            ));
        }
        if self.max_zoom > 20 {
            return Err(GeoArrowError::Serialization(
                "max_zoom cannot exceed 20".to_string(),
            ));
        }
        Ok(())
    }

    pub fn is_visible_at_zoom(&self, zoom: u8) -> bool {
        self.visible && zoom >= self.min_zoom && zoom <= self.max_zoom
    }

    pub fn with_style(mut self, style: LayerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_zoom_range(mut self, min_zoom: u8, max_zoom: u8) -> Self {
        self.min_zoom = min_zoom.min(20);
        self.max_zoom = max_zoom.min(20);
        self
    }
}

// Viewport for map view management
#[derive(Clone, Debug)]
pub struct Viewport {
    pub center: GeoPoint,
    pub zoom: f64,
    pub rotation: f64,
    pub size: PixelSize,
    pub bounds: GeoBounds,
    pub pixel_bounds: PixelBounds,
}

impl Viewport {
    pub fn new(center: GeoPoint, zoom: f64, size: PixelSize) -> GeoArrowResult<Self> {
        if !center.is_valid() {
            return Err(GeoArrowError::Serialization(
                "Invalid center coordinates".to_string(),
            ));
        }
        if zoom < 0.0 || zoom > 20.0 {
            return Err(GeoArrowError::Serialization(
                "Zoom must be between 0.0 and 20.0".to_string(),
            ));
        }
        if size.width == 0 || size.height == 0 {
            return Err(GeoArrowError::Serialization(
                "Size dimensions must be greater than 0".to_string(),
            ));
        }

        let mut viewport = Viewport {
            center,
            zoom,
            rotation: 0.0,
            size: size.clone(),
            bounds: GeoBounds::new(0.0, 0.0, 0.0, 0.0),
            pixel_bounds: PixelBounds::new(0.0, 0.0, size.width as f64, size.height as f64),
        };

        viewport.recalculate_bounds();
        Ok(viewport)
    }

    pub fn pan(&mut self, new_center: GeoPoint) -> GeoArrowResult<()> {
        if !new_center.is_valid() {
            return Err(GeoArrowError::Serialization(
                "Invalid center coordinates".to_string(),
            ));
        }
        self.center = new_center;
        self.recalculate_bounds();
        Ok(())
    }

    pub fn zoom_to(&mut self, new_zoom: f64) -> GeoArrowResult<()> {
        if new_zoom < 0.0 || new_zoom > 20.0 {
            return Err(GeoArrowError::Serialization(
                "Zoom must be between 0.0 and 20.0".to_string(),
            ));
        }
        self.zoom = new_zoom;
        self.recalculate_bounds();
        Ok(())
    }

    pub fn resize(&mut self, new_size: PixelSize) -> GeoArrowResult<()> {
        if new_size.width == 0 || new_size.height == 0 {
            return Err(GeoArrowError::Serialization(
                "Size dimensions must be greater than 0".to_string(),
            ));
        }
        self.size = new_size.clone();
        self.pixel_bounds =
            PixelBounds::new(0.0, 0.0, new_size.width as f64, new_size.height as f64);
        self.recalculate_bounds();
        Ok(())
    }

    pub fn rotate(&mut self, rotation: f64) {
        self.rotation = rotation;
        self.recalculate_bounds();
    }

    fn recalculate_bounds(&mut self) {
        // Calculate the geographic bounds based on center, zoom, and size
        // This is a simplified calculation for Web Mercator projection
        let scale = 1.0 / (1u32 << self.zoom as u32) as f64;
        let half_width = (self.size.width as f64 / 2.0) * scale * 360.0 / 256.0;
        let half_height = (self.size.height as f64 / 2.0) * scale * 180.0 / 256.0;

        self.bounds = GeoBounds::new(
            self.center.lng - half_width,
            self.center.lat - half_height,
            self.center.lng + half_width,
            self.center.lat + half_height,
        );
    }

    pub fn world_to_screen(&self, point: &GeoPoint) -> (f64, f64) {
        if self.bounds.is_empty() {
            return (0.0, 0.0);
        }

        let x_ratio = (point.lng - self.bounds.min_x) / (self.bounds.max_x - self.bounds.min_x);
        let y_ratio = (point.lat - self.bounds.min_y) / (self.bounds.max_y - self.bounds.min_y);

        let screen_x = x_ratio * self.size.width as f64;
        let screen_y = self.size.height as f64 - (y_ratio * self.size.height as f64); // Flip Y axis

        (screen_x, screen_y)
    }

    pub fn screen_to_world(&self, x: f64, y: f64) -> GeoPoint {
        if self.bounds.is_empty() {
            return self.center.clone();
        }

        let x_ratio = x / self.size.width as f64;
        let y_ratio = (self.size.height as f64 - y) / self.size.height as f64; // Flip Y axis

        let lng = self.bounds.min_x + x_ratio * (self.bounds.max_x - self.bounds.min_x);
        let lat = self.bounds.min_y + y_ratio * (self.bounds.max_y - self.bounds.min_y);

        GeoPoint::new(lat, lng)
    }

    pub fn get_required_tiles(&self) -> Vec<(u32, u32, u8)> {
        let z = self.zoom.floor() as u8;
        if z > 20 {
            return Vec::new();
        }

        let tile_count = 1u32 << z;

        // Calculate tile bounds
        let min_tile_x = ((self.bounds.min_x + 180.0) / 360.0 * tile_count as f64).floor() as u32;
        let max_tile_x = ((self.bounds.max_x + 180.0) / 360.0 * tile_count as f64).ceil() as u32;
        let min_tile_y =
            ((1.0 - (self.bounds.max_y + 90.0) / 180.0) * tile_count as f64).floor() as u32;
        let max_tile_y =
            ((1.0 - (self.bounds.min_y + 90.0) / 180.0) * tile_count as f64).ceil() as u32;

        let mut tiles = Vec::new();
        for x in min_tile_x..=max_tile_x.min(tile_count - 1) {
            for y in min_tile_y..=max_tile_y.min(tile_count - 1) {
                tiles.push((x, y, z));
            }
        }
        tiles
    }
}
