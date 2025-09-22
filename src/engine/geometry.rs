use geojson::{Geometry, Value as GeoValue, Position};
use crate::engine::RenderContext;

// Pure geometry transformation functions

// Higher-order coordinate transformation
pub const transform_coordinates: fn(&RenderContext) -> fn(&[Position]) -> Vec<(f64, f64)> =
    |context| |positions| {
        positions.iter()
            .map(|pos| transform_position(context, pos))
            .collect()
    };

// Transform single position to screen coordinates
pub const transform_position: fn(&RenderContext, &Position) -> (f64, f64) =
    |context, position| {
        let x = position[0];
        let y = position[1];
        (crate::engine::RenderContext::world_to_screen)(context, x, y)
    };

// Geometry-specific transformers
pub const extract_point_coordinates: fn(&Geometry) -> Option<Vec<Position>> =
    |geometry| {
        match &geometry.value {
            GeoValue::Point(coords) => Some(vec![coords.clone()]),
            _ => None,
        }
    };

pub const extract_linestring_coordinates: fn(&Geometry) -> Option<Vec<Position>> =
    |geometry| {
        match &geometry.value {
            GeoValue::LineString(coords) => Some(coords.clone()),
            _ => None,
        }
    };

pub const extract_polygon_coordinates: fn(&Geometry) -> Option<Vec<Vec<Position>>> =
    |geometry| {
        match &geometry.value {
            GeoValue::Polygon(rings) => Some(rings.clone()),
            _ => None,
        }
    };

pub const extract_multipoint_coordinates: fn(&Geometry) -> Option<Vec<Position>> =
    |geometry| {
        match &geometry.value {
            GeoValue::MultiPoint(points) => Some(points.clone()),
            _ => None,
        }
    };

pub const extract_multilinestring_coordinates: fn(&Geometry) -> Option<Vec<Vec<Position>>> =
    |geometry| {
        match &geometry.value {
            GeoValue::MultiLineString(lines) => Some(lines.clone()),
            _ => None,
        }
    };

pub const extract_multipolygon_coordinates: fn(&Geometry) -> Option<Vec<Vec<Vec<Position>>>> =
    |geometry| {
        match &geometry.value {
            GeoValue::MultiPolygon(polygons) => Some(polygons.clone()),
            _ => None,
        }
    };

// Utility functions for coordinate validation and bounds checking
pub const validate_coordinates: fn(&[(f64, f64)]) -> bool =
    |coords| coords.iter().all(|(x, y)| x.is_finite() && y.is_finite());

pub const filter_coordinates_in_bounds: fn(&[(f64, f64)], &RenderContext) -> Vec<(f64, f64)> =
    |coords, context| {
        coords.iter()
            .filter(|(x, y)| {
                *x >= 0.0 && *x <= context.canvas_size.0 &&
                *y >= 0.0 && *y <= context.canvas_size.1
            })
            .cloned()
            .collect()
    };

// Coordinate transformation pipeline
pub const create_coordinate_transformer: fn(&RenderContext) -> fn(&Geometry) -> Option<Vec<(f64, f64)>> =
    |context| |geometry| {
        let transform_coords = transform_coordinates(context);

        match &geometry.value {
            GeoValue::Point(_) =>
                extract_point_coordinates(geometry).map(|coords| transform_coords(&coords)),
            GeoValue::LineString(_) =>
                extract_linestring_coordinates(geometry).map(|coords| transform_coords(&coords)),
            GeoValue::MultiPoint(_) =>
                extract_multipoint_coordinates(geometry).map(|coords| transform_coords(&coords)),
            _ => None,
        }
    };

// Polygon-specific transformer (returns outer ring only for simplicity)
pub const create_polygon_transformer: fn(&RenderContext) -> fn(&Geometry) -> Option<Vec<(f64, f64)>> =
    |context| |geometry| {
        let transform_coords = transform_coordinates(context);

        extract_polygon_coordinates(geometry)
            .and_then(|rings| rings.first().map(|outer_ring| transform_coords(outer_ring)))
    };