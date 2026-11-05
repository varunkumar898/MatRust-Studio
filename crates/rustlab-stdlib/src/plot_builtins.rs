//! Plot builtins: figure, plot, scatter, bar, hist, hold, subplot,
//! xlabel, ylabel, title, legend, xlim, ylim, grid, saveas.
//!
//! Figure state lives in `rustlab-plot::PlotRegistry` (thread-local) and
//! mirrors MATLAB's `gcf`/`gca` semantics within one interpreter session.

use crate::helpers::{extract_f64_array, extract_string, math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::Value;
use rustlab_plot::{
    to_svg, with_plot_state_ref, Axes, Bar, Figure, Histogram, Legend, LegendLocation, Line,
    Scatter, Series, Shading,
};

/// `figure` — create a new figure or switch to an existing one.
/// `h = figure` creates a new figure and returns its handle.
/// `figure(h)` switches to figure `h` and returns `h`.
pub fn figure_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let id = match args.len() {
        0 => rustlab_plot::with_plot_state(|reg| reg.create_figure()),
        1 => {
            let id = args[0].as_scalar_i64().map_err(math_err)? as usize;
            let set = rustlab_plot::with_plot_state(|reg| reg.set_current(id));
            if !set {
                return Err(user_err(
                    "RUSTLAB:plot:noFigure",
                    &format!("Figure {id} does not exist."),
                ));
            }
            id
        }
        _ => {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "figure accepts 0 or 1 inputs.",
            ));
        }
    };
    Ok(vec![Value::scalar(id as f64)])
}

/// Ensure the current figure has at least one axes; create default axes if
/// missing (MATLAB creates axes on first plot).
fn ensure_current_axes(fig: &mut Figure) -> Result<&mut Axes, EvalError> {
    if fig.axes.is_empty() {
        let ax = Axes::new(1);
        fig.current_axes = Some(1);
        fig.axes.push(ax);
    }
    let ax_id = fig
        .current_axes
        .ok_or(user_err("RUSTLAB:plot:noFigure", "No current axes."))?;
    fig.axes
        .iter_mut()
        .find(|a| a.id == ax_id)
        .ok_or(user_err("RUSTLAB:plot:noFigure", "No current axes."))
}

/// `plot(x, y)` — add a line series to the current axes.
pub fn plot_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 2 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "plot requires exactly 2 inputs (x, y).",
        ));
    }
    let x = extract_f64_array(&args[0])?;
    let y = extract_f64_array(&args[1])?;
    if x.as_slice().len() != y.as_slice().len() {
        return Err(user_err(
            "RUSTLAB:plot:emptyData",
            "X and Y vectors must have the same length.",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.push_series(Series::Line(Line::new(
            "",
            x.as_slice().to_vec(),
            y.as_slice().to_vec(),
        )));
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `scatter(x, y)` — add a scatter series to the current axes.
pub fn scatter_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 2 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "scatter requires exactly 2 inputs (x, y).",
        ));
    }
    let x = extract_f64_array(&args[0])?;
    let y = extract_f64_array(&args[1])?;
    if x.as_slice().len() != y.as_slice().len() {
        return Err(user_err(
            "RUSTLAB:plot:emptyData",
            "X and Y vectors must have the same length.",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.push_series(Series::Scatter(Scatter::new(
            "",
            x.as_slice().to_vec(),
            y.as_slice().to_vec(),
        )));
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `bar(x, y)` — add a bar series to the current axes.
pub fn bar_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 2 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "bar requires exactly 2 inputs (x, y).",
        ));
    }
    let x = extract_f64_array(&args[0])?;
    let y = extract_f64_array(&args[1])?;
    if x.as_slice().len() != y.as_slice().len() {
        return Err(user_err(
            "RUSTLAB:plot:emptyData",
            "X and Y vectors must have the same length.",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.push_series(Series::Bar(Bar::new(
            "",
            x.as_slice().to_vec(),
            y.as_slice().to_vec(),
        )));
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `hist(y, nbins)` — add a histogram series to the current axes.
pub fn hist_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() || args.len() > 2 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "hist requires 1 or 2 inputs (y, nbins).",
        ));
    }
    let y = extract_f64_array(&args[0])?;
    let bins = if args.len() == 2 {
        args[1].as_scalar_i64().map_err(math_err)? as usize
    } else {
        10
    };
    if bins == 0 {
        return Err(user_err(
            "RUSTLAB:hist:badBins",
            "Histogram bin count must be positive.",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.push_series(Series::Histogram(Histogram::new(
            "",
            y.as_slice().to_vec(),
            bins,
        )));
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `hold` — toggle hold on current axes.
/// `hold('on'|'off')` — set hold on current axes.
/// `hold(ax, 'on'|'off')` — set hold on axes `ax` of the current figure.
pub fn hold_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    match args.len() {
        0 => {
            // Toggle current axes hold.
            rustlab_plot::with_plot_state(|reg| {
                let fig = reg.current_figure_mut().ok_or(user_err(
                    "RUSTLAB:plot:noFigure",
                    "No current figure. Call figure first.",
                ))?;
                let ax = ensure_current_axes(fig)?;
                ax.hold = !ax.hold;
                Ok::<(), EvalError>(())
            })?;
        }
        1 => {
            let state = extract_string(&args[0]).map_err(|_| {
                user_err(
                    "RUSTLAB:plot:badArgs",
                    "hold requires 1 input ('on' or 'off').",
                )
            })?;
            let on = match state.as_str() {
                "on" => true,
                "off" => false,
                _ => {
                    return Err(user_err(
                        "RUSTLAB:plot:badArgs",
                        "hold state must be 'on' or 'off'.",
                    ));
                }
            };
            rustlab_plot::with_plot_state(|reg| {
                let fig = reg.current_figure_mut().ok_or(user_err(
                    "RUSTLAB:plot:noFigure",
                    "No current figure. Call figure first.",
                ))?;
                let ax = ensure_current_axes(fig)?;
                ax.hold = on;
                Ok::<(), EvalError>(())
            })?;
        }
        2 => {
            let ax_id = args[0].as_scalar_i64().map_err(math_err)? as usize;
            let state = extract_string(&args[1]).map_err(|_| {
                user_err(
                    "RUSTLAB:plot:badArgs",
                    "hold(ax, state) requires state 'on' or 'off'.",
                )
            })?;
            let on = match state.as_str() {
                "on" => true,
                "off" => false,
                _ => {
                    return Err(user_err(
                        "RUSTLAB:plot:badArgs",
                        "hold state must be 'on' or 'off'.",
                    ));
                }
            };
            rustlab_plot::with_plot_state(|reg| {
                let fig = reg.current_figure_mut().ok_or(user_err(
                    "RUSTLAB:plot:noFigure",
                    "No current figure. Call figure first.",
                ))?;
                let ax = fig.axes.iter_mut().find(|a| a.id == ax_id).ok_or(user_err(
                    "RUSTLAB:plot:noAxes",
                    &format!("Axes {ax_id} does not exist."),
                ))?;
                ax.hold = on;
                Ok::<(), EvalError>(())
            })?;
        }
        _ => {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "hold accepts 0, 1, or 2 inputs.",
            ));
        }
    }
    Ok(vec![])
}

/// `ishhold()` — return whether hold is on for the current axes.
/// `ishhold(ax)` — return whether hold is on for axes `ax` of the current figure.
pub fn ishold_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let hold = match args.len() {
        0 => rustlab_plot::with_plot_state_ref(|reg| {
            let fig = reg.current_figure().ok_or_else(|| {
                user_err(
                    "RUSTLAB:plot:noFigure",
                    "No current figure. Call figure first.",
                )
            })?;
            let ax = fig
                .current_axes()
                .ok_or_else(|| user_err("RUSTLAB:plot:noAxes", "No current axes."))?;
            Ok::<bool, EvalError>(ax.hold)
        })?,
        1 => {
            let ax_id = args[0].as_scalar_i64().map_err(math_err)? as usize;
            rustlab_plot::with_plot_state_ref(|reg| {
                let fig = reg.current_figure().ok_or_else(|| {
                    user_err(
                        "RUSTLAB:plot:noFigure",
                        "No current figure. Call figure first.",
                    )
                })?;
                let ax = fig.axes.iter().find(|a| a.id == ax_id).ok_or_else(|| {
                    user_err(
                        "RUSTLAB:plot:noAxes",
                        &format!("Axes {ax_id} does not exist."),
                    )
                })?;
                Ok::<bool, EvalError>(ax.hold)
            })?
        }
        _ => {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "ishold accepts 0 or 1 inputs.",
            ));
        }
    };
    Ok(vec![Value::logical_scalar(hold)])
}

/// `gcf` — return the id of the current figure.
pub fn gcf_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if !args.is_empty() {
        return Err(user_err("RUSTLAB:plot:badArgs", "gcf accepts no inputs."));
    }
    let id = rustlab_plot::with_plot_state_ref(|reg| {
        reg.current_id.ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))
    })?;
    Ok(vec![Value::scalar(id as f64)])
}

/// `gca` — return the id of the current axes.
pub fn gca_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if !args.is_empty() {
        return Err(user_err("RUSTLAB:plot:badArgs", "gca accepts no inputs."));
    }
    let ax_id = rustlab_plot::with_plot_state_ref(|reg| {
        let fig = reg.current_figure().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        fig.current_axes()
            .map(|a| a.id)
            .ok_or(user_err("RUSTLAB:plot:noAxes", "No current axes."))
    })?;
    Ok(vec![Value::scalar(ax_id as f64)])
}

/// `cla` — clear the current axes (remove all series).
pub fn cla_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if !args.is_empty() {
        return Err(user_err("RUSTLAB:plot:badArgs", "cla accepts no inputs."));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.series.clear();
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `clf` — clear the current figure (remove all axes).
pub fn clf_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if !args.is_empty() {
        return Err(user_err("RUSTLAB:plot:badArgs", "clf accepts no inputs."));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        fig.axes.clear();
        fig.current_axes = None;
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `close` — close the current figure, figure `n`, or all figures (`'all'`).
pub fn close_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    match args.len() {
        0 => {
            rustlab_plot::with_plot_state(|reg| {
                let id = reg
                    .current_id
                    .ok_or(user_err("RUSTLAB:plot:noFigure", "No current figure."))?;
                reg.figures.remove(&id);
                if reg.current_id == Some(id) {
                    reg.current_id = reg.figure_ids().first().copied();
                }
                Ok::<(), EvalError>(())
            })?;
        }
        1 => {
            if let Ok(s) = extract_string(&args[0]) {
                if s == "all" {
                    rustlab_plot::with_plot_state(|reg| {
                        reg.figures.clear();
                        reg.current_id = None;
                        Ok::<(), EvalError>(())
                    })?;
                } else {
                    return Err(user_err(
                        "RUSTLAB:plot:badArgs",
                        "close accepts a figure id or 'all'.",
                    ));
                }
            } else {
                let id = args[0].as_scalar_i64().map_err(math_err)? as usize;
                rustlab_plot::with_plot_state(|reg| {
                    reg.figures.remove(&id);
                    if reg.current_id == Some(id) {
                        reg.current_id = reg.figure_ids().first().copied();
                    }
                    Ok::<(), EvalError>(())
                })?;
            }
        }
        _ => {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "close accepts 0 or 1 inputs.",
            ));
        }
    }
    Ok(vec![])
}

/// `zlabel(text)` — set z-axis label on the current axes.
pub fn zlabel_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "zlabel requires exactly 1 input.",
        ));
    }
    let text = extract_string(&args[0])
        .map_err(|_| user_err("RUSTLAB:plot:badArgs", "zlabel requires 1 input (text)."))?;
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.zlabel = text;
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `subplot(m, n, p)` — create axes in an m×n grid at position p (1-based).
/// `subplot(mnp)` — three-digit form, e.g. `subplot(211)` == `subplot(2,1,1)`.
pub fn subplot_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    let (m, n, p) = if args.len() == 1 {
        let mnp = args[0].as_scalar_i64().map_err(math_err)? as usize;
        if !(100..=999).contains(&mnp) {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "subplot(mnp) requires a 3-digit number.",
            ));
        }
        (mnp / 100, (mnp / 10) % 10, mnp % 10)
    } else if args.len() == 3 {
        (
            args[0].as_scalar_i64().map_err(math_err)? as usize,
            args[1].as_scalar_i64().map_err(math_err)? as usize,
            args[2].as_scalar_i64().map_err(math_err)? as usize,
        )
    } else {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "subplot requires 3 inputs (m, n, p) or 1 input (mnp).",
        ));
    };
    if m == 0 || n == 0 || p == 0 || p > m * n {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "Invalid subplot dimensions.",
        ));
    }
    let m = args[0].as_scalar_i64().map_err(math_err)? as usize;
    let n = args[1].as_scalar_i64().map_err(math_err)? as usize;
    let p = args[2].as_scalar_i64().map_err(math_err)? as usize;
    if m == 0 || n == 0 || p == 0 || p > m * n {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "Invalid subplot dimensions.",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        // Reuse existing axes with id p if present.
        if fig.axes.iter().any(|a| a.id == p) {
            fig.current_axes = Some(p);
            return Ok::<(), EvalError>(());
        }
        let mut ax = Axes::new(p);
        ax.title = format!("subplot({m},{n},{p})");
        fig.add_axes(ax);
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `xlabel(text)` — set x-axis label on current axes.
pub fn xlabel_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "xlabel requires exactly 1 input.",
        ));
    }
    let text = extract_string(&args[0])?;
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.xlabel = text;
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `ylabel(text)` — set y-axis label on current axes.
pub fn ylabel_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "ylabel requires exactly 1 input.",
        ));
    }
    let text = extract_string(&args[0])?;
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.ylabel = text;
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `title(text)` — set figure title.
pub fn title_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "title requires exactly 1 input.",
        ));
    }
    let text = extract_string(&args[0])?;
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        fig.title = text;
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `legend(loc)` — set legend location on current axes.
pub fn legend_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "legend requires exactly 1 input (location).",
        ));
    }
    let loc = extract_string(&args[0])?;
    let location = match loc.as_str() {
        "northeast" => LegendLocation::NorthEast,
        "northwest" => LegendLocation::NorthWest,
        "southeast" => LegendLocation::SouthEast,
        "southwest" => LegendLocation::SouthWest,
        _ => {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "legend location must be 'northeast', 'northwest', 'southeast', or 'southwest'.",
            ));
        }
    };
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.legend = Some(Legend { location });
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `xlim([lo hi])` — set x-axis limits on current axes.
pub fn xlim_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "xlim requires exactly 1 input ([lo hi]).",
        ));
    }
    let arr = extract_f64_array(&args[0])?;
    let slice = arr.as_slice();
    if slice.len() != 2 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "xlim requires a 2-element vector [lo hi].",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.xlim = Some([slice[0], slice[1]]);
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `ylim([lo hi])` — set y-axis limits on current axes.
pub fn ylim_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "ylim requires exactly 1 input ([lo hi]).",
        ));
    }
    let arr = extract_f64_array(&args[0])?;
    let slice = arr.as_slice();
    if slice.len() != 2 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "ylim requires a 2-element vector [lo hi].",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.ylim = Some([slice[0], slice[1]]);
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `grid` — toggle grid on current axes.
/// `grid('on'|'off')` — set grid state.
pub fn grid_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() > 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "grid accepts 0 or 1 inputs ('on' or 'off').",
        ));
    }
    if args.is_empty() {
        rustlab_plot::with_plot_state(|reg| {
            let fig = reg.current_figure_mut().ok_or(user_err(
                "RUSTLAB:plot:noFigure",
                "No current figure. Call figure first.",
            ))?;
            let ax = ensure_current_axes(fig)?;
            ax.grid = !ax.grid;
            Ok::<(), EvalError>(())
        })?;
        return Ok(Vec::new());
    }
    let state = extract_string(&args[0])?;
    let on = match state.as_str() {
        "on" => true,
        "off" => false,
        _ => {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "grid state must be 'on' or 'off'.",
            ));
        }
    };
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.grid = on;
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `saveas(handle, 'file.svg')` or `saveas('file.svg')` — write SVG to disk.
pub fn saveas_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() || args.len() > 2 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "saveas requires 1 or 2 inputs (handle, filename).",
        ));
    }
    let (fig_id, filename) = if args.len() == 2 {
        let id = args[0].as_scalar_i64().map_err(math_err)? as usize;
        let name = extract_string(&args[1])?;
        (id, name)
    } else {
        let name = extract_string(&args[0])?;
        // Find current figure id.
        let current = rustlab_plot::with_plot_state_ref(|reg| reg.current_id);
        match current {
            Some(id) => (id, name),
            None => {
                return Err(user_err(
                    "RUSTLAB:plot:noFigure",
                    "No current figure. Call figure first.",
                ));
            }
        }
    };

    let svg = with_plot_state_ref(|reg| {
        let fig = reg.get_figure(fig_id).ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            &format!("Figure {fig_id} does not exist."),
        ))?;
        Ok::<String, EvalError>(to_svg(fig, 640.0, 480.0))
    })?;

    if std::fs::write(&filename, &svg).is_err() {
        return Err(user_err(
            "RUSTLAB:plot:saveFail",
            &format!("Could not write '{filename}'."),
        ));
    }
    Ok(vec![])
}

/// `surf(X, Y, Z)` — create a 3D surface plot from grid matrices.
/// `surf(Z)` — uses implicit X = 1:cols, Y = 1:rows.
pub fn surf_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.is_empty() || args.len() > 3 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "surf requires 1 to 3 inputs (X, Y, Z) or just Z.",
        ));
    }
    let (x_vec, y_vec, z_arr) = if args.len() == 1 {
        // Implicit X = 1:cols, Y = 1:rows.
        let z_arr = args[0].to_double_array().map_err(math_err)?;
        let shape = z_arr.shape();
        if shape.len() != 2 {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "surf(Z) requires a 2-D matrix Z.",
            ));
        }
        let cols = shape[1];
        let rows = shape[0];
        let x_vec: Vec<f64> = (1..=cols).map(|i| i as f64).collect();
        let y_vec: Vec<f64> = (1..=rows).map(|i| i as f64).collect();
        (x_vec, y_vec, z_arr)
    } else if args.len() == 3 {
        let x_arr = args[0].to_double_array().map_err(math_err)?;
        let y_arr = args[1].to_double_array().map_err(math_err)?;
        let z_arr = args[2].to_double_array().map_err(math_err)?;
        let x_slice = x_arr.as_slice();
        let y_slice = y_arr.as_slice();
        let z_slice = z_arr.as_slice();
        if x_slice.len() != y_slice.len() || x_slice.len() != z_slice.len() {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "surf(X, Y, Z) requires X, Y, Z to have the same number of elements.",
            ));
        }
        (x_slice.to_vec(), y_slice.to_vec(), z_arr)
    } else {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "surf requires 1 or 3 inputs.",
        ));
    };

    let z_slice = z_arr.as_slice();
    let shape = z_arr.shape();
    let rows = shape[0];
    let cols = shape[1];
    // Only the explicit (X, Y, Z) form requires X and Y to be full grids of
    // Z's size; the implicit `surf(Z)` form uses plain coordinate vectors.
    let grid = args.len() == 3;
    if grid && (x_vec.len() != rows * cols || y_vec.len() != rows * cols) {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "surf(X, Y, Z) requires X and Y to be Z's size (rows*cols).",
        ));
    }

    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or(user_err(
            "RUSTLAB:plot:noFigure",
            "No current figure. Call figure first.",
        ))?;
        let ax = ensure_current_axes(fig)?;
        ax.push_series(rustlab_plot::Series::Surface(rustlab_plot::Surface::new(
            "surface",
            rows,
            cols,
            x_vec,
            y_vec,
            z_slice.to_vec(),
        )));
        ax.colorbar = Some(rustlab_plot::Colorbar {
            location: rustlab_plot::ColorbarLocation::East,
            label: "Z".to_string(),
        });
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `colorbar` — show the colorbar for the current axes.
/// `colorbar(ax)` — show the colorbar for axes `ax` of the current figure.
pub fn colorbar_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() > 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "colorbar accepts 0 or 1 inputs.",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err(
                "RUSTLAB:plot:noFigure",
                "No current figure. Call figure first.",
            )
        })?;
        let ax: &mut Axes = if args.is_empty() {
            ensure_current_axes(fig)?
        } else {
            let id = args[0].as_scalar_i64().map_err(math_err)? as usize;
            fig.axes.iter_mut().find(|a| a.id == id).ok_or_else(|| {
                user_err("RUSTLAB:plot:noAxes", &format!("Axes {id} does not exist."))
            })?
        };
        if ax.colorbar.is_none() {
            ax.colorbar = Some(rustlab_plot::Colorbar {
                location: rustlab_plot::ColorbarLocation::East,
                label: "Z".to_string(),
            });
        }
        Ok::<(), EvalError>(())
    })?;
    Ok(Vec::new())
}

/// `colormap(name)` — set the colormap of the current axes.
/// `colormap` — return the name of the current colormap.
pub fn colormap_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() > 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "colormap accepts 0 or 1 inputs.",
        ));
    }
    if args.is_empty() {
        let name = rustlab_plot::with_plot_state_ref(|reg| {
            let fig = reg.current_figure().ok_or_else(|| {
                user_err(
                    "RUSTLAB:plot:noFigure",
                    "No current figure. Call figure first.",
                )
            })?;
            let ax = fig
                .current_axes()
                .ok_or_else(|| user_err("RUSTLAB:plot:noAxes", "No current axes."))?;
            Ok::<&'static str, EvalError>(colormap_name(ax.colormap))
        })?;
        return Ok(vec![Value::string_scalar(name)]);
    }
    let name = extract_string(&args[0])?;
    let cmap = match name.as_str() {
        "rustlab" => rustlab_plot::Colormap::Rustlab,
        _ => {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                &format!("Unknown colormap '{name}'. Supported: rustlab."),
            ));
        }
    };
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err(
                "RUSTLAB:plot:noFigure",
                "No current figure. Call figure first.",
            )
        })?;
        let ax = ensure_current_axes(fig)?;
        ax.colormap = cmap;
        Ok::<(), EvalError>(())
    })?;
    Ok(Vec::new())
}

fn colormap_name(c: rustlab_plot::Colormap) -> &'static str {
    match c {
        rustlab_plot::Colormap::Rustlab => "rustlab",
    }
}

/// `shading(mode)` — set surface shading on the current axes.
/// `mode` is 'flat' (default) or 'interp'.
pub fn shading_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "shading requires 1 input ('flat' or 'interp').",
        ));
    }
    let mode = extract_string(&args[0])?;
    let shading = match mode.as_str() {
        "flat" => Shading::Flat,
        "interp" => Shading::Interp,
        _ => {
            return Err(user_err(
                "RUSTLAB:plot:badArgs",
                "shading mode must be 'flat' or 'interp'.",
            ));
        }
    };
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err(
                "RUSTLAB:plot:noFigure",
                "No current figure. Call figure first.",
            )
        })?;
        let ax = ensure_current_axes(fig)?;
        ax.shading = shading;
        Ok::<(), EvalError>(())
    })?;
    Ok(Vec::new())
}

/// `view(az, el)` — set the 3-D camera azimuth and elevation in degrees.
pub fn view_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 2 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "view requires 2 inputs (azimuth, elevation).",
        ));
    }
    let azimuth = args[0].as_scalar_f64().map_err(math_err)?;
    let elevation = args[1].as_scalar_f64().map_err(math_err)?;
    if !azimuth.is_finite() || !elevation.is_finite() {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "view angles must be finite scalars.",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err(
                "RUSTLAB:plot:noFigure",
                "No current figure. Call figure first.",
            )
        })?;
        let ax = ensure_current_axes(fig)?;
        ax.camera = rustlab_plot::Camera {
            azimuth,
            elevation,
        };
        Ok::<(), EvalError>(())
    })?;
    Ok(Vec::new())
}

/// `axis(mode)` — set axis scaling: 'auto', 'tight', or 'equal'.
/// `axis([lo hi])` — set the x-axis limits of the current axes.
pub fn axis_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "axis requires 1 input (mode or [lo hi]).",
        ));
    }
    if let Ok(mode) = extract_string(&args[0]) {
        rustlab_plot::with_plot_state(|reg| {
            let fig = reg.current_figure_mut().ok_or_else(|| {
                user_err(
                    "RUSTLAB:plot:noFigure",
                    "No current figure. Call figure first.",
                )
            })?;
            let ax = ensure_current_axes(fig)?;
            match mode.as_str() {
                "auto" => {
                    ax.xlim = None;
                    ax.ylim = None;
                    ax.zlim = None;
                }
                "tight" => {
                    if let Some((xr, yr, zr)) = ax.data_bounds() {
                        ax.xlim = Some(xr);
                        ax.ylim = Some(yr);
                        if ax.series.iter().any(|s| matches!(s, Series::Surface(_))) {
                            ax.zlim = Some(zr);
                        }
                    }
                }
                "equal" => {
                    if let Some((xr, yr, _zr)) = ax.data_bounds() {
                        let xspan = xr[1] - xr[0];
                        let yspan = yr[1] - yr[0];
                        let span = xspan.max(yspan).max(1e-12);
                        let xc = 0.5 * (xr[0] + xr[1]);
                        let yc = 0.5 * (yr[0] + yr[1]);
                        ax.xlim = Some([xc - 0.5 * span, xc + 0.5 * span]);
                        ax.ylim = Some([yc - 0.5 * span, yc + 0.5 * span]);
                    }
                }
                _ => {
                    return Err(user_err(
                        "RUSTLAB:plot:badArgs",
                        "axis mode must be 'auto', 'tight', or 'equal'.",
                    ));
                }
            }
            Ok::<(), EvalError>(())
        })?;
        return Ok(Vec::new());
    }
    let arr = extract_f64_array(&args[0])?;
    let slice = arr.as_slice();
    if slice.len() != 2 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "axis requires a mode string or a 2-element vector [lo hi].",
        ));
    }
    rustlab_plot::with_plot_state(|reg| {
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err(
                "RUSTLAB:plot:noFigure",
                "No current figure. Call figure first.",
            )
        })?;
        let ax = ensure_current_axes(fig)?;
        ax.xlim = Some([slice[0], slice[1]]);
        Ok::<(), EvalError>(())
    })?;
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustlab_math::DenseArray;

    #[test]
    fn figure_create_and_switch() {
        rustlab_plot::plot_state_reset();
        let result = figure_fn(&[], 1).unwrap();
        assert_eq!(result[0], Value::scalar(1.0));
        let result = figure_fn(&[], 1).unwrap();
        assert_eq!(result[0], Value::scalar(2.0));
        // Switch back to figure 1
        let result = figure_fn(&[Value::scalar(1.0)], 1).unwrap();
        assert_eq!(result[0], Value::scalar(1.0));
    }

    #[test]
    fn plot_stores_series() {
        rustlab_plot::plot_state_reset();
        figure_fn(&[], 1).unwrap();
        let x = Value::Double(DenseArray::from_vec(vec![1, 3], vec![1.0, 2.0, 3.0]).unwrap());
        let y = Value::Double(DenseArray::from_vec(vec![1, 3], vec![4.0, 5.0, 6.0]).unwrap());
        plot_fn(&[x, y], 0).unwrap();
        let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
        let ax = fig.current_axes().unwrap();
        assert_eq!(ax.series.len(), 1);
    }

    #[test]
    fn hold_toggles() {
        rustlab_plot::plot_state_reset();
        figure_fn(&[], 1).unwrap();
        hold_fn(&[Value::string_scalar("off")], 0).unwrap();
        let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
        assert!(!fig.current_axes().unwrap().hold);
    }

    #[test]
    fn surf_stores_surface_series() {
        rustlab_plot::plot_state_reset();
        figure_fn(&[], 1).unwrap();
        let z = Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 2.0, 3.0, 4.0]).unwrap());
        surf_fn(&[z], 0).unwrap();
        let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
        let ax = fig.current_axes().unwrap();
        assert_eq!(ax.series.len(), 1);
        assert!(matches!(ax.series[0], rustlab_plot::Series::Surface(_)));
    }

    #[test]
    fn surf_adds_colorbar() {
        rustlab_plot::plot_state_reset();
        figure_fn(&[], 1).unwrap();
        let z = Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 2.0, 3.0, 4.0]).unwrap());
        surf_fn(&[z], 0).unwrap();
        let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
        let ax = fig.current_axes().unwrap();
        assert!(ax.colorbar.is_some());
    }
}
