use crate::model::Bounds;

// Pure transformation functions for coordinate systems and projections

// Zoom transformation functions
pub const apply_zoom_transform: fn(f64, f64, u8) -> (f64, f64) =
    |x, y, zoom_level| {
        let scale_factor = 2.0_f64.powi(zoom_level as i32);
        (x * scale_factor, y * scale_factor)
    };

pub const create_zoom_transformer: fn(u8) -> fn(f64, f64) -> (f64, f64) =
    |zoom_level| |x, y| apply_zoom_transform(x, y, zoom_level);

// Viewport bounds calculations
pub const calculate_viewport_bounds: fn((f64, f64), (f64, f64), u8) -> Bounds =
    |center, viewport_size, zoom_level| {
        let (center_x, center_y) = center;
        let (width, height) = viewport_size;
        let scale = 2.0_f64.powi(-(zoom_level as i32));

        let half_width = (width * scale) / 2.0;
        let half_height = (height * scale) / 2.0;

        Bounds::new(
            center_x - half_width,
            center_y - half_height,
            center_x + half_width,
            center_y + half_height,
        )
    };

// Bounds utility functions
pub const expand_bounds: fn(&Bounds, f64) -> Bounds =
    |bounds, factor| {
        let width = bounds.max_x - bounds.min_x;
        let height = bounds.max_y - bounds.min_y;
        let expand_x = width * factor / 2.0;
        let expand_y = height * factor / 2.0;

        Bounds::new(
            bounds.min_x - expand_x,
            bounds.min_y - expand_y,
            bounds.max_x + expand_x,
            bounds.max_y + expand_y,
        )
    };

pub const bounds_center: fn(&Bounds) -> (f64, f64) =
    |bounds| {
        let center_x = (bounds.min_x + bounds.max_x) / 2.0;
        let center_y = (bounds.min_y + bounds.max_y) / 2.0;
        (center_x, center_y)
    };

pub const bounds_size: fn(&Bounds) -> (f64, f64) =
    |bounds| (bounds.max_x - bounds.min_x, bounds.max_y - bounds.min_y);

// Aspect ratio preservation
pub const preserve_aspect_ratio: fn(&Bounds, f64) -> Bounds =
    |bounds, target_aspect_ratio| {
        let (width, height) = bounds_size(bounds);
        let current_aspect = width / height;

        if current_aspect > target_aspect_ratio {
            // Bounds too wide, expand height
            let new_height = width / target_aspect_ratio;
            let height_diff = new_height - height;
            Bounds::new(
                bounds.min_x,
                bounds.min_y - height_diff / 2.0,
                bounds.max_x,
                bounds.max_y + height_diff / 2.0,
            )
        } else {
            // Bounds too tall, expand width
            let new_width = height * target_aspect_ratio;
            let width_diff = new_width - width;
            Bounds::new(
                bounds.min_x - width_diff / 2.0,
                bounds.min_y,
                bounds.max_x + width_diff / 2.0,
                bounds.max_y,
            )
        }
    };

// Pan transformation functions
pub const apply_pan_transform: fn(f64, f64, f64, f64) -> (f64, f64) =
    |x, y, dx, dy| (x + dx, y + dy);

pub const create_pan_transformer: fn(f64, f64) -> fn(f64, f64) -> (f64, f64) =
    |dx, dy| |x, y| apply_pan_transform(x, y, dx, dy);

// Screen to world coordinate transformation
pub const screen_to_world: fn((f64, f64), (f64, f64), &Bounds) -> (f64, f64) =
    |screen_pos, canvas_size, bounds| {
        let (screen_x, screen_y) = screen_pos;
        let (canvas_width, canvas_height) = canvas_size;

        let x_ratio = screen_x / canvas_width;
        let y_ratio = 1.0 - (screen_y / canvas_height); // Flip Y axis

        let world_x = bounds.min_x + x_ratio * (bounds.max_x - bounds.min_x);
        let world_y = bounds.min_y + y_ratio * (bounds.max_y - bounds.min_y);

        (world_x, world_y)
    };

// Bounding box calculations from coordinates
pub const calculate_bounds_from_coordinates: fn(&[(f64, f64)]) -> Option<Bounds> =
    |coordinates| {
        if coordinates.is_empty() {
            return None;
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for &(x, y) in coordinates {
            if x < min_x { min_x = x; }
            if y < min_y { min_y = y; }
            if x > max_x { max_x = x; }
            if y > max_y { max_y = y; }
        }

        Some(Bounds::new(min_x, min_y, max_x, max_y))
    };

// Fit bounds to viewport
pub const fit_bounds_to_viewport: fn(&Bounds, (f64, f64)) -> (u8, (f64, f64)) =
    |data_bounds, viewport_size| {
        let (data_width, data_height) = bounds_size(data_bounds);
        let (viewport_width, viewport_height) = viewport_size;

        let scale_x = viewport_width / data_width;
        let scale_y = viewport_height / data_height;
        let scale = scale_x.min(scale_y);

        // Calculate zoom level (rough approximation)
        let zoom_level = (scale.log2().floor() as i32).max(1).min(20) as u8;

        let center = bounds_center(data_bounds);

        (zoom_level, center)
    };