//! Plot builtins: figure, plot, scatter, bar, hist, hold, subplot,
//! xlabel, ylabel, title, legend, xlim, ylim, grid, saveas.
//!
//! Figure state lives in `rustlab-plot::PlotRegistry` (thread-local) and
//! mirrors MATLAB's `gcf`/`gca` semantics within one interpreter session.

use crate::helpers::{extract_f64_array, extract_string, math_err, user_err};
use rustlab_eval::EvalError;
use rustlab_math::Value;
use rustlab_plot::{Axes, Bar, Figure, Histogram, Legend, LegendLocation, Line, Scatter, Series, to_svg, with_plot_state_ref};

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
    let ax_id = fig.current_axes.ok_or_else(|| {
        user_err("RUSTLAB:plot:noFigure", "No current axes.")
    })?;
    fig.axes
        .iter_mut()
        .find(|a| a.id == ax_id)
        .ok_or_else(|| user_err("RUSTLAB:plot:noFigure", "No current axes."))
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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

/// `hold('on'|'off')` — toggle hold on current axes.
pub fn hold_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "hold requires 1 input ('on' or 'off').",
        ));
    }
    let state = extract_string(&args[0])?;
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
        let ax = ensure_current_axes(fig)?;
        ax.hold = on;
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `subplot(m, n, p)` — create axes in an m×n grid at position p (1-based).
pub fn subplot_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 3 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "subplot requires 3 inputs (m, n, p).",
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
        // Reuse existing axes with id p if present.
        if fig.axes.iter().any(|a| a.id == p) {
            fig.current_axes = Some(p);
            return Ok::<(), EvalError>(())
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
        let ax = ensure_current_axes(fig)?;
        ax.ylim = Some([slice[0], slice[1]]);
        Ok::<(), EvalError>(())
    })?;
    Ok(vec![])
}

/// `grid('on'|'off')` — toggle grid on current axes.
pub fn grid_fn(args: &[Value], _nargout: usize) -> Result<Vec<Value>, EvalError> {
    if args.len() != 1 {
        return Err(user_err(
            "RUSTLAB:plot:badArgs",
            "grid requires 1 input ('on' or 'off').",
        ));
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
        let fig = reg.current_figure_mut().ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", "No current figure. Call figure first.")
        })?;
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
        let fig = reg.get_figure(fig_id).ok_or_else(|| {
            user_err("RUSTLAB:plot:noFigure", &format!("Figure {fig_id} does not exist."))
        })?;
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
}
