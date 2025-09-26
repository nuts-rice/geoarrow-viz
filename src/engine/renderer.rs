use web_sys::CanvasRenderingContext2d;
use crate::engine::RenderContext;
use crate::model::GeoArrowResult;
use crate::error::GeoArrowError;

// Higher-order rendering functions

// Canvas setup function
pub const setup_canvas_context: fn(&CanvasRenderingContext2d, &RenderContext) -> GeoArrowResult<()> =
    |context, render_context| {
        let style = &render_context.style;

        context.set_fill_style_str(&style.polygon_fill);
        context.set_stroke_style_str(&style.polygon_stroke);
        context.set_line_width(style.line_width);

        Ok(())
    };

// Point rendering functions
pub const render_points: fn(&[(f64, f64)], &RenderContext, &CanvasRenderingContext2d) -> GeoArrowResult<()> =
    |points, render_context, canvas_context| {
        let style = &render_context.style;

        canvas_context.set_fill_style_str(&style.point_color);

        points.iter()
            .map(|(x, y)| render_single_point(*x, *y, style.point_radius, canvas_context))
            .collect::<Result<Vec<_>, _>>()
            .map(|_| ())
    };

pub const render_single_point: fn(f64, f64, f64, &CanvasRenderingContext2d) -> GeoArrowResult<()> =
    |x, y, radius, context| {
        context.begin_path();
        context.arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI)
            .map_err(|_| GeoArrowError::Wasm("Failed to draw arc".to_string()))?;
        context.fill();
        Ok(())
    };

// Line rendering functions
pub const render_linestring: fn(&[(f64, f64)], &RenderContext, &CanvasRenderingContext2d) -> GeoArrowResult<()> =
    |points, render_context, canvas_context| {
        if points.is_empty() {
            return Ok(());
        }

        let style = &render_context.style;
        canvas_context.set_stroke_style_str(&style.line_color);
        canvas_context.set_line_width(style.line_width);

        draw_path(points, canvas_context)?;
        canvas_context.stroke();
        Ok(())
    };

// Polygon rendering functions
pub const render_polygon: fn(&[(f64, f64)], &RenderContext, &CanvasRenderingContext2d) -> GeoArrowResult<()> =
    |points, render_context, canvas_context| {
        if points.is_empty() {
            return Ok(());
        }

        let style = &render_context.style;
        canvas_context.set_fill_style_str(&style.polygon_fill);
        canvas_context.set_stroke_style_str(&style.polygon_stroke);
        canvas_context.set_line_width(style.line_width);

        draw_path(points, canvas_context)?;
        canvas_context.close_path();
        canvas_context.fill();
        canvas_context.stroke();
        Ok(())
    };



// Utility path drawing function
pub const draw_path: fn(&[(f64, f64)], &CanvasRenderingContext2d) -> GeoArrowResult<()> =
    |points, context| {
        if let Some((first_x, first_y)) = points.first() {
            context.begin_path();
            context.move_to(*first_x, *first_y);

            points.iter().skip(1)
                .try_for_each(|(x, y)| {
                    context.line_to(*x, *y);
                    Ok::<(), GeoArrowError>(())
                })?;
        }
        Ok(())
    };

// Rendering function combinators
pub const compose_renderers: fn(
    fn(&[(f64, f64)], &RenderContext, &CanvasRenderingContext2d) -> GeoArrowResult<()>,
    fn(&[(f64, f64)], &RenderContext, &CanvasRenderingContext2d) -> GeoArrowResult<()>
) -> fn(&[(f64, f64)], &RenderContext, &CanvasRenderingContext2d) -> GeoArrowResult<()> =
    |renderer1, renderer2| |points, render_context, canvas_context| {
        renderer1(points, render_context, canvas_context)?;
        renderer2(points, render_context, canvas_context)
    };

// Clear canvas function
pub const clear_canvas: fn(&CanvasRenderingContext2d, (f64, f64)) -> GeoArrowResult<()> =
    |context, (width, height)| {
        context.clear_rect(0.0, 0.0, width, height);
        Ok(())
    };

// Background drawing function
pub const draw_background: fn(&CanvasRenderingContext2d, (f64, f64), &str) -> GeoArrowResult<()> =
    |context, (width, height), color| {
        context.set_fill_style_str(color);
        context.fill_rect(0.0, 0.0, width, height);
        Ok(())
    };

// Grid drawing function (for debugging/reference)
pub const draw_grid: fn(&CanvasRenderingContext2d, (f64, f64), f64) -> GeoArrowResult<()> =
    |context, (width, height), spacing| {
        context.set_stroke_style_str("#cccccc");
        context.set_line_width(0.5);

        // Vertical lines
        let mut x = 0.0;
        while x <= width {
            context.begin_path();
            context.move_to(x, 0.0);
            context.line_to(x, height);
            context.stroke();
            x += spacing;
        }

        // Horizontal lines
        let mut y = 0.0;
        while y <= height {
            context.begin_path();
            context.move_to(0.0, y);
            context.line_to(width, y);
            context.stroke();
            y += spacing;
        }

        Ok(())
    };
