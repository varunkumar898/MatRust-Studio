//! Deterministic SVG renderer for `Figure`.
//!
//! Output uses fixed 2-decimal coordinate formatting so identical inputs
//! produce byte-identical strings across runs and platforms. Element ids
//! are derived from stable figure/axes/series indices (not hashes or
//! pointers). No timestamps, no randomness, no external renderer.

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

        // Axis limits: explicit xlim/ylim, else data bounds.
        let (x_min, x_max, y_min, y_max) = axis_limits(axes);

        let ctx = RenderContext {
            px0,
            py0,
            pw,
            ph,
            x_min,
            x_max,
            y_min,
            y_max,
        };

        // X axis line + ticks.
        if axes.grid {
            out.push_str(&grid_lines(&ctx, axes));
        }

        // Series.
        for series in &axes.series {
            out.push_str(&render_series(series, &ctx));
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
.legend { fill:#ffffff; stroke:#999999; stroke-width:1; }
.legend-text { font-size: 10px; fill:#222222; }
</style></defs>"#
}

/// Resolve explicit limits or fall back to data bounds, with a small
/// margin so points are not clipped exactly on the edge.
fn axis_limits(axes: &Axes) -> (f64, f64, f64, f64) {
    let (x_min, x_max) = match axes.xlim {
        Some([lo, hi]) => (lo, hi),
        None => match axes.data_bounds() {
            Some((xr, _)) => margin(xr[0], xr[1]),
            None => (0.0, 1.0),
        },
    };
    let (y_min, y_max) = match axes.ylim {
        Some([lo, hi]) => (lo, hi),
        None => match axes.data_bounds() {
            Some((_, yr)) => margin(yr[0], yr[1]),
            None => (0.0, 1.0),
        },
    };
    (x_min, x_max, y_min, y_max)
}

fn margin(lo: f64, hi: f64) -> (f64, f64) {
    if (hi - lo).abs() < 1e-12 {
        let half = if lo.abs() < 1e-12 { 0.5 } else { (hi - lo).abs() * 0.05 + 0.5 };
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

/// Map data y to pixel y (flipped: larger y = higher on screen).
fn py(y: f64, y_min: f64, y_max: f64, origin: f64, height: f64) -> f64 {
    if (y_max - y_min).abs() < 1e-12 {
        return origin + height / 2.0;
    }
    origin + height - ((y - y_min) / (y_max - y_min)) * height
}

/// Bundles the plot box and axis limits so render helpers take few args.
struct RenderContext {
    px0: f64,
    py0: f64,
    pw: f64,
    ph: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

fn grid_lines(ctx: &RenderContext, _axes: &Axes) -> String {
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

fn render_series(series: &crate::Series, ctx: &RenderContext) -> String {
    match series {
        crate::Series::Line(l) => {
            if l.x.len() < 2 || l.x.len() != l.y.len() {
                return String::new();
            }
            let mut pts = String::new();
            for (x, y) in l.x.iter().zip(l.y.iter()) {
                pts.push_str(&format!("{},{} ", fmt2(px(*x, ctx.x_min, ctx.x_max, ctx.px0, ctx.pw)), fmt2(py(*y, ctx.y_min, ctx.y_max, ctx.py0, ctx.ph))));
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
                    color, fmt2(cx), fmt2(cy), r
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
    }
}

fn rgb255(c: [f32; 4]) -> String {
    let r = (c[0].clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (c[1].clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (c[2].clamp(0.0, 1.0) * 255.0).round() as u32;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_is_deterministic() {
        let mut fig = Figure::new(1);
        let mut ax = Axes::new(1);
        ax.push_series(crate::Series::Line(crate::Line::new("l", vec![1.0, 2.0, 3.0], vec![2.0, 4.0, 6.0])));
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
        ax.push_series(crate::Series::Line(crate::Line::new("s", vec![0.0], vec![0.0])));
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
}
