//! Deterministic SVG renderer for `Figure`.
//!
//! Output uses fixed 2-decimal coordinate formatting so identical inputs
//! produce byte-identical strings across runs and platforms. Element ids
//! are derived from stable figure/axes/series indices (not hashes or
//! pointers). No timestamps, no randomness, no external renderer.
//!
//! Supports 2D series (Line, Scatter, Bar, Histogram) and 3D Surface
//! rendering with orthographic projection, painter's-algorithm depth sort,
//! and a colormap-driven colorbar.

use crate::Axes;
use crate::Figure;

/// Render a figure to an SVG string of the given pixel size.
pub fn to_svg(fig: &Figure, width: f64, height: f64) -> String {
    let mut out = String::new();
    out.push_str(&svg_header(width, height));
    out.push_str(&style_block());

    let pad = 50.0;
    let plot_w = width - 2.0 * pad;
    let plot_h = height - 2.0 * pad;

    // Title (figure-level).
    if !fig.title.is_empty() {
        out.push_str(&format!(
            r#"<text x="{}" y="28" class="fig-title" text-anchor="middle">{}</text>"#,
            fmt2(width / 2.0),
            esc(&fig.title)
        ));
    }

    for axes in &fig.axes {
        let subpad = pad + 20.0;
        let sub_w = plot_w - 20.0;
        let sub_h = plot_h - 40.0;
        if fig.axes.len() > 1 {
            out.push_str(&format!(
                r#"<g id="axes-{}"><rect x="{}" y="{}" width="{}" height="{}" class="axes-bg"/>"#,
                axes.id,
                fmt2(subpad),
                fmt2(60.0),
                fmt2(sub_w),
                fmt2(sub_h)
            ));
        } else {
            out.push_str(&format!(
                r#"<g id="axes-{}"><rect x="{}" y="{}" width="{}" height="{}" class="axes-bg"/>"#,
                axes.id,
                fmt2(pad),
                fmt2(60.0),
                fmt2(plot_w),
                fmt2(plot_h)
            ));
        }

        // Determine plot area.
        let (px0, py0, pw, ph) = if fig.axes.len() > 1 {
            (subpad, 60.0, sub_w, sub_h)
        } else {
            (pad, 60.0, plot_w, plot_h)
        };

        // Axis limits: explicit xlim/ylim/zlim, else data bounds.
        let db = axes.data_bounds();
        let (x_min, x_max) = match axes.xlim {
            Some([lo, hi]) => (lo, hi),
            None => match db {
                Some((xr, _, _)) => margin(xr[0], xr[1]),
                None => (0.0, 1.0),
            },
        };
        let (y_min, y_max) = match axes.ylim {
            Some([lo, hi]) => (lo, hi),
            None => match db {
                Some((_, yr, _)) => margin(yr[0], yr[1]),
                None => (0.0, 1.0),
            },
        };
        let (z_min, z_max) = match axes.zlim {
            Some([lo, hi]) => (lo, hi),
            None => match db {
                Some((_, _, zr)) => margin(zr[0], zr[1]),
                None => (0.0, 1.0),
            },
        };

        let ctx = RenderContext {
            px0,
            py0,
            pw,
            ph,
            axes,
            x_min,
            x_max,
            y_min,
            y_max,
            _z_min: z_min,
            _z_max: z_max,
            _has_surface: axes
                .series
                .iter()
                .any(|s| matches!(s, crate::Series::Surface(_))),
        };

        // X axis line + ticks.
        if axes.grid {
            out.push_str(&grid_lines(&ctx));
        }

        // Series.
        for series in &axes.series {
            out.push_str(&render_series(series, &ctx));
        }

        // Colorbar (drawn after series so it sits on top).
        if let Some(cb) = &axes.colorbar {
            out.push_str(&render_colorbar(&ctx, cb));
        }

        out.push_str("</g>\n");
    }

    out.push_str("</svg>");
    out
}

/// Fixed 2-decimal number formatting.
fn fmt2(v: f64) -> String {
    // Round to 2 decimals then format; guarantees stable text.
    let r = (v * 100.0).round() / 100.0;
    format!("{:.2}", r)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn svg_header(width: f64, height: f64) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}" class="rustlab-plot">"#,
        fmt2(width),
        fmt2(height),
        fmt2(width),
        fmt2(height)
    )
}

fn style_block() -> &'static str {
    r#"<defs><style>
.rustlab-plot { background:#ffffff; font-family: sans-serif; font-size: 11px; }
.axes-bg { fill:#ffffff; stroke:#cccccc; stroke-width:1; }
.grid-line { stroke:#eaeaea; stroke-width:1; }
.axis-line { stroke:#333333; stroke-width:1; }
.fig-title { fill:#111111; font-weight: bold; font-size: 14px; }
.axes-label { fill:#333333; font-size: 11px; text-anchor: middle; }
.series-line { fill:none; stroke-width:2; }
.series-scatter { fill:#0072BD; }
.series-bar { fill:#0072BD; }
.series-surface { stroke:#333333; stroke-width:0.5; }
.colorbar { stroke:#333333; stroke-width:1; }
.colorbar-label { font-size: 10px; fill:#222222; text-anchor: middle; }
.colorbar-tick { font-size: 9px; fill:#222222; text-anchor: end; }
.legend { fill:#ffffff; stroke:#999999; stroke-width:1; }
.legend-text { font-size: 10px; fill:#222222; }
</style></defs>"#
}

/// Resolve explicit limits or fall back to data bounds, with a small
/// margin so points are not clipped exactly on the edge.
fn axis_limits(axes: &Axes) -> (f64, f64, f64, f64, f64, f64) {
    let (x_min, x_max) = match axes.xlim {
        Some([lo, hi]) => (lo, hi),
        None => match axes.data_bounds() {
            Some((xr, _, _)) => margin(xr[0], xr[1]),
            None => (0.0, 1.0),
        },
    };
    let (y_min, y_max) = match axes.ylim {
        Some([lo, hi]) => (lo, hi),
        None => match axes.data_bounds() {
            Some((_, yr, _)) => margin(yr[0], yr[1]),
            None => (0.0, 1.0),
        },
    };
    let (z_min, z_max) = match axes.zlim {
        Some([lo, hi]) => (lo, hi),
        None => match axes.data_bounds() {
            Some((_, _, zr)) => margin(zr[0], zr[1]),
            None => (0.0, 1.0),
        },
    };
    (x_min, x_max, y_min, y_max, z_min, z_max)
}

fn margin(lo: f64, hi: f64) -> (f64, f64) {
    if (hi - lo).abs() < 1e-12 {
        let half = if lo.abs() < 1e-12 {
            0.5
        } else {
            (hi - lo).abs() * 0.05 + 0.5
        };
        (lo - half, hi + half)
    } else {
        let pad = (hi - lo) * 0.05;
        (lo - pad, hi + pad)
    }
}

/// Map data x to pixel x within the plot box.
fn px(x: f64, x_min: f64, x_max: f64, origin: f64, width: f64) -> f64 {
    if (x_max - x_min).abs() < 1e-12 {
        return origin + width / 2.0;
    }
    origin + ((x - x_min) / (x_max - x_min)) * width
}

/// Map data y to pixel y within the plot box (flipped: larger y = higher on screen).
fn py(y: f64, y_min: f64, y_max: f64, origin: f64, height: f64) -> f64 {
    if (y_max - y_min).abs() < 1e-12 {
        return origin + height / 2.0;
    }
    origin + height - ((y - y_min) / (y_max - y_min)) * height
}

/// Bundles the plot box and axis limits so render helpers take few args.
struct RenderContext<'a> {
    px0: f64,
    py0: f64,
    pw: f64,
    ph: f64,
    axes: &'a Axes,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    _z_min: f64,
    _z_max: f64,
    _has_surface: bool,
}

/// A projected 3D point: screen x/y and depth for painter's-algorithm sort.
#[derive(Debug, Clone, Copy)]
struct ProjPt {
    sx: f64,
    sy: f64,
    depth: f64,
}

/// Project a 3D point (x, y, z) onto the 2D screen using orthographic
/// projection with the axes' camera (azimuth / elevation).
/// Returns screen coordinates and a depth value: larger depth = farther
/// from the viewer (drawn first in painter's algorithm).
#[allow(clippy::too_many_arguments)]
fn project3d(
    x: f64,
    y: f64,
    z: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64,
    cam: &crate::Camera,
    origin: f64,
    width: f64,
    height: f64,
) -> ProjPt {
    #[allow(clippy::too_many_arguments)]
    fn project3d_inner(
        x: f64,
        y: f64,
        z: f64,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        z_min: f64,
        z_max: f64,
        cam: &crate::Camera,
        origin: f64,
        width: f64,
        height: f64,
    ) -> ProjPt {
        // Normalize to [-1, 1] cube.
        let xn = if (x_max - x_min).abs() < 1e-12 {
            0.0
        } else {
            2.0 * (x - x_min) / (x_max - x_min) - 1.0
        };
        let yn = if (y_max - y_min).abs() < 1e-12 {
            0.0
        } else {
            2.0 * (y - y_min) / (y_max - y_min) - 1.0
        };
        let zn = if (z_max - z_min).abs() < 1e-12 {
            0.0
        } else {
            2.0 * (z - z_min) / (z_max - z_min) - 1.0
        };

        let az = cam.azimuth.to_radians();
        let el = cam.elevation.to_radians();
        let (ca, sa) = (az.cos(), az.sin());
        let (ce, se) = (el.cos(), el.sin());

        // Rotate around Z by azimuth.
        let x1 = xn * ca - yn * sa;
        let y1 = xn * sa + yn * ca;
        let z1 = zn;

        // Tilt by elevation: screen_y combines y1 (depth) and z1 (height).
        let sx = x1;
        let sy = y1 * se - z1 * ce;
        let depth = y1 * ce + z1 * se; // larger = farther from viewer

        ProjPt {
            sx: origin + (sx + 1.0) / 2.0 * width,
            sy: origin + height - (sy + 1.0) / 2.0 * height,
            depth,
        }
    }
    project3d_inner(x, y, z, x_min, x_max, y_min, y_max, z_min, z_max, cam, origin, width, height)
}

/// Render colorbar for the axes.
fn render_colorbar(ctx: &RenderContext, cb: &crate::Colorbar) -> String {
    let (_x_min, _x_max, _y_min, _y_max, z_min, z_max) = axis_limits(ctx.axes);
    let cb_w = 14.0;
    let cb_h = ctx.ph * 0.6;
    let cb_x = ctx.px0 + ctx.pw + 10.0;
    let cb_y = ctx.py0 + (ctx.ph - cb_h) / 2.0;

    let mut out = String::new();
    // Colorbar background frame.
    out.push_str(&format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" class="colorbar" fill="none" />"#,
        fmt2(cb_x),
        fmt2(cb_y),
        fmt2(cb_w),
        fmt2(cb_h)
    ));

    // Vertical gradient: sample colormap at 20 bands.
    let n_bands = 20;
    for i in 0..n_bands {
        let t0 = i as f64 / n_bands as f64;
        let t1 = (i + 1) as f64 / n_bands as f64;
        let _c0 = crate::colormap_lookup(ctx.axes.colormap, t0);
        let _c1 = crate::colormap_lookup(ctx.axes.colormap, t1);
        let mid_t = (t0 + t1) / 2.0;
        let cm = crate::colormap_lookup(ctx.axes.colormap, mid_t);
        let y_top = cb_y + (1.0 - t1) * cb_h;
        let y_bot = cb_y + (1.0 - t0) * cb_h;
        out.push_str(&format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"#{:02x}{:02x}{:02x}\" />",
            fmt2(cb_x),
            fmt2(y_top),
            fmt2(cb_w),
            fmt2(y_bot - y_top),
            (cm[0] * 255.0).round() as u32,
            (cm[1] * 255.0).round() as u32,
            (cm[2] * 255.0).round() as u32
        ));
    }

    // Tick labels: min, mid, max.
    let ticks = [z_min, (z_min + z_max) / 2.0, z_max];
    for tv in ticks {
        let t = if (z_max - z_min).abs() < 1e-12 {
            0.5
        } else {
            (tv - z_min) / (z_max - z_min)
        };
        let ty = cb_y + (1.0 - t) * cb_h;
        out.push_str(&format!(
            r#"<text x="{}" y="{}" class="colorbar-tick">{}</text>"#,
            fmt2(cb_x - 2.0),
            fmt2(ty + 3.0),
            fmt2(tv)
        ));
    }

    // Colorbar label.
    if !cb.label.is_empty() {
        out.push_str(&format!(
            r#"<text x="{}" y="{}" class="colorbar-label">{}</text>"#,
            fmt2(cb_x + cb_w / 2.0),
            fmt2(cb_y + cb_h + 16.0),
            esc(&cb.label)
        ));
    }

    out
}

/// Render one series, dispatching by type.
fn render_series(series: &crate::Series, ctx: &RenderContext) -> String {
    match series {
        crate::Series::Line(l) => {
            if l.x.len() < 2 || l.x.len() != l.y.len() {
                return String::new();
            }
            let mut pts = String::new();
            for (x, y) in l.x.iter().zip(l.y.iter()) {
                pts.push_str(&format!(
                    "{},{} ",
                    fmt2(px(*x, ctx.x_min, ctx.x_max, ctx.px0, ctx.pw)),
                    fmt2(py(*y, ctx.y_min, ctx.y_max, ctx.py0, ctx.ph))
                ));
            }
            let color = rgb255(l.color);
            format!(
                r#"<polyline class="series-line" stroke="{}" stroke-width="{}" points="{}" />"#,
                color,
                fmt2(l.width as f64),
                pts.trim_end()
            )
        }
        crate::Series::Scatter(s) => {
            if s.x.len() != s.y.len() {
                return String::new();
            }
            let mut out = String::new();
            let color = rgb255(s.color);
            let r = fmt2((s.size / 2.0).into());
            for (x, y) in s.x.iter().zip(s.y.iter()) {
                let cx = px(*x, ctx.x_min, ctx.x_max, ctx.px0, ctx.pw);
                let cy = py(*y, ctx.y_min, ctx.y_max, ctx.py0, ctx.ph);
                out.push_str(&format!(
                    r#"<circle class="series-scatter" fill="{}" cx="{}" cy="{}" r="{}" />"#,
                    color,
                    fmt2(cx),
                    fmt2(cy),
                    r
                ));
            }
            out
        }
        crate::Series::Bar(b) => {
            if b.x.len() != b.y.len() || b.x.is_empty() {
                return String::new();
            }
            let color = rgb255(b.color);
            let n = b.x.len() as f64;
            let bar_w = ctx.pw / n * 0.7;
            let mut out = String::new();
            for (i, (_x, y)) in b.x.iter().zip(b.y.iter()).enumerate() {
                let bx = ctx.px0 + (i as f64 + 0.15) / n * ctx.pw;
                let by = py(*y, ctx.y_min, ctx.y_max, ctx.py0, ctx.ph);
                let top = ctx.py0 + ctx.ph;
                out.push_str(&format!(
                    r#"<rect class="series-bar" fill="{}" x="{}" y="{}" width="{}" height="{}" />"#,
                    color,
                    fmt2(bx),
                    fmt2(by),
                    fmt2(bar_w),
                    fmt2(top - by)
                ));
            }
            out
        }
        crate::Series::Histogram(h) => {
            if h.values.is_empty() {
                return String::new();
            }
            let vmin = h.values.iter().cloned().fold(f64::INFINITY, f64::min);
            let vmax = h.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let (bin_lo, bin_hi) = if (vmax - vmin).abs() < 1e-12 {
                (vmin - 0.5, vmax + 0.5)
            } else {
                (vmin, vmax)
            };
            let bin_w = (bin_hi - bin_lo) / h.bins as f64;
            let mut counts = vec![0usize; h.bins];
            for v in &h.values {
                let idx = (((v - bin_lo) / bin_w) * h.bins as f64).floor() as usize;
                let idx = idx.min(h.bins - 1);
                counts[idx] += 1;
            }
            let max_count = counts.iter().copied().max().unwrap_or(1) as f64;
            let color = rgb255(h.color);
            let mut out = String::new();
            for (i, &c) in counts.iter().enumerate() {
                if c == 0 {
                    continue;
                }
                let bx = ctx.px0 + (i as f64 / h.bins as f64) * ctx.pw;
                let bw = ctx.pw / h.bins as f64 * 0.9;
                let bh = (c as f64 / max_count) * ctx.ph * 0.75;
                let by = ctx.py0 + ctx.ph - bh;
                out.push_str(&format!(
                    r#"<rect class="series-bar" fill="{}" x="{}" y="{}" width="{}" height="{}" />"#,
                    color,
                    fmt2(bx),
                    fmt2(by),
                    fmt2(bw),
                    fmt2(bh)
                ));
            }
            out
        }
        crate::Series::Surface(s) => {
            render_surface(s, ctx)
        }
    }
}

/// Render a surface series as a mesh of quads with painter's-algorithm
/// depth sort and colormap shading.
fn render_surface(s: &crate::Surface, ctx: &RenderContext) -> String {
    if s.rows < 2 || s.cols < 2 {
        return String::new();
    }
    let (x_min, x_max, y_min, y_max, z_min, z_max) = axis_limits(ctx.axes);

    // Build projected quad list with depth for sort.
    let mut quads: Vec<(f64, Vec<ProjPt>, f64)> = Vec::new(); // (depth, pts, avg_z)
    for j in 0..s.rows - 1 {
        for i in 0..s.cols - 1 {
            let idx = |i: usize, j: usize| i + j * s.cols;
            let pts = [
                project3d(
                    s.x[idx(i, j)], s.y[idx(i, j)], s.z[idx(i, j)],
                    x_min, x_max, y_min, y_max, z_min, z_max,
                    &ctx.axes.camera, ctx.px0, ctx.pw, ctx.ph,
                ),
                project3d(
                    s.x[idx(i + 1, j)], s.y[idx(i + 1, j)], s.z[idx(i + 1, j)],
                    x_min, x_max, y_min, y_max, z_min, z_max,
                    &ctx.axes.camera, ctx.px0, ctx.pw, ctx.ph,
                ),
                project3d(
                    s.x[idx(i + 1, j + 1)], s.y[idx(i + 1, j + 1)], s.z[idx(i + 1, j + 1)],
                    x_min, x_max, y_min, y_max, z_min, z_max,
                    &ctx.axes.camera, ctx.px0, ctx.pw, ctx.ph,
                ),
                project3d(
                    s.x[idx(i, j + 1)], s.y[idx(i, j + 1)], s.z[idx(i, j + 1)],
                    x_min, x_max, y_min, y_max, z_min, z_max,
                    &ctx.axes.camera, ctx.px0, ctx.pw, ctx.ph,
                ),
            ];
            let depth = pts.iter().map(|p| p.depth).sum::<f64>() / 4.0;
            let avg_z = (s.z[idx(i, j)] + s.z[idx(i + 1, j)] + s.z[idx(i + 1, j + 1)] + s.z[idx(i, j + 1)]) / 4.0;
            quads.push((depth, pts.to_vec(), avg_z));
        }
    }

    // Painter's algorithm: draw farthest quads first.
    quads.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut out = String::new();
    for (_depth, pts, avg_z) in quads {
        let t = if (z_max - z_min).abs() < 1e-12 {
            0.5
        } else {
            (avg_z - z_min) / (z_max - z_min)
        };
        let cm = crate::colormap_lookup(ctx.axes.colormap, t);
        let color = format!(
            "#{:02x}{:02x}{:02x}",
            (cm[0] * 255.0).round() as u32,
            (cm[1] * 255.0).round() as u32,
            (cm[2] * 255.0).round() as u32
        );
        out.push_str(&format!(
            r#"<polygon class="series-surface" fill="{}" points="{},{} {},{} {},{} {},{}" />"#,
            color,
            fmt2(pts[0].sx), fmt2(pts[0].sy),
            fmt2(pts[1].sx), fmt2(pts[1].sy),
            fmt2(pts[2].sx), fmt2(pts[2].sy),
            fmt2(pts[3].sx), fmt2(pts[3].sy),
        ));
    }
    out
}

fn rgb255(c: [f32; 4]) -> String {
    let r = (c[0].clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (c[1].clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (c[2].clamp(0.0, 1.0) * 255.0).round() as u32;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Render grid lines for 2D view.
fn grid_lines(ctx: &RenderContext) -> String {
    let mut out = String::new();
    let xticks = crate::ticks::nice_ticks(ctx.x_min, ctx.x_max, 6);
    let yticks = crate::ticks::nice_ticks(ctx.y_min, ctx.y_max, 5);
    for tx in &xticks {
        let x = px(*tx, ctx.x_min, ctx.x_max, ctx.px0, ctx.pw);
        out.push_str(&format!(
            r#"<line class="grid-line" x1="{}" y1="{}" x2="{}" y2="{}" />"#,
            fmt2(x),
            fmt2(ctx.py0),
            fmt2(x),
            fmt2(ctx.py0 + ctx.ph)
        ));
    }
    for ty in &yticks {
        let y = py(*ty, ctx.y_min, ctx.y_max, ctx.py0, ctx.ph);
        out.push_str(&format!(
            r#"<line class="grid-line" x1="{}" y1="{}" x2="{}" y2="{}" />"#,
            fmt2(ctx.px0),
            fmt2(y),
            fmt2(ctx.px0 + ctx.pw),
            fmt2(y)
        ));
    }
    // Axis frame.
    out.push_str(&format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" class="axis-line" fill="none" />"#,
        fmt2(ctx.px0),
        fmt2(ctx.py0),
        fmt2(ctx.pw),
        fmt2(ctx.ph)
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_is_deterministic() {
        let mut fig = Figure::new(1);
        let mut ax = Axes::new(1);
        ax.push_series(crate::Series::Line(crate::Line::new(
            "l",
            vec![1.0, 2.0, 3.0],
            vec![2.0, 4.0, 6.0],
        )));
        fig.add_axes(ax);
        let a = to_svg(&fig, 400.0, 300.0);
        let b = to_svg(&fig, 400.0, 300.0);
        assert_eq!(a, b);
        assert!(a.contains("<svg"));
        assert!(a.contains("</svg>"));
    }

    #[test]
    fn svg_has_stable_ids() {
        let mut fig = Figure::new(7);
        let mut ax = Axes::new(3);
        ax.push_series(crate::Series::Line(crate::Line::new(
            "s",
            vec![0.0],
            vec![0.0],
        )));
        fig.add_axes(ax);
        let svg = to_svg(&fig, 400.0, 300.0);
        assert!(svg.contains(r#"id="axes-3""#));
    }

    #[test]
    fn fmt2_is_stable() {
        let f = fmt2(1.005);
        assert!(f == "1.00" || f == "1.01", "unexpected fmt2: {f}");
        assert_eq!(fmt2(0.0), "0.00");
        assert_eq!(fmt2(-1.5), "-1.50");
    }

    #[test]
    fn rustlab_colormap_is_in_range() {
        for i in 0..=100 {
            let t = i as f64 / 100.0;
            let c = crate::colormap_lookup(crate::Colormap::Rustlab, t);
            assert!(c[0] >= 0.0 && c[0] <= 1.0, "r out of range at t={t}");
            assert!(c[1] >= 0.0 && c[1] <= 1.0, "g out of range at t={t}");
            assert!(c[2] >= 0.0 && c[2] <= 1.0, "b out of range at t={t}");
        }
    }

    #[test]
    fn surface_renders_deterministically() {
        let mut fig = Figure::new(1);
        let mut ax = Axes::new(1);
        let rows = 4;
        let cols = 4;
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut z = Vec::new();
        for j in 0..rows {
            for i in 0..cols {
                x.push(i as f64);
                y.push(j as f64);
                z.push((i as f64 * j as f64) as f64);
            }
        }
        ax.push_series(crate::Series::Surface(crate::Surface::new(
            "surf", rows, cols, x, y, z,
        )));
        fig.add_axes(ax);
        let a = to_svg(&fig, 400.0, 300.0);
        let b = to_svg(&fig, 400.0, 300.0);
        assert_eq!(a, b);
        assert!(a.contains("polygon"));
        assert!(a.contains("series-surface"));
    }

    #[test]
    fn colorbar_renders() {
        let mut fig = Figure::new(1);
        let mut ax = Axes::new(1);
        ax.colorbar = Some(crate::Colorbar {
            location: crate::ColorbarLocation::East,
            label: "Z values".to_string(),
        });
        let rows = 3;
        let cols = 3;
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut z = Vec::new();
        for j in 0..rows {
            for i in 0..cols {
                x.push(i as f64);
                y.push(j as f64);
                z.push((i + j) as f64);
            }
        }
        ax.push_series(crate::Series::Surface(crate::Surface::new(
            "surf", rows, cols, x, y, z,
        )));
        fig.add_axes(ax);
        let svg = to_svg(&fig, 400.0, 300.0);
        assert!(svg.contains("colorbar"));
        assert!(svg.contains("Z values"));
    }

    #[test]
    fn project3d_is_deterministic() {
        let cam = crate::Camera::default();
        let p1 = project3d(0.0, 0.0, 0.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, &cam, 0.0, 100.0, 100.0);
        let p2 = project3d(0.0, 0.0, 0.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, &cam, 0.0, 100.0, 100.0);
        assert_eq!(p1.sx, p2.sx);
        assert_eq!(p1.sy, p2.sy);
        assert_eq!(p1.depth, p2.depth);
    }
}
