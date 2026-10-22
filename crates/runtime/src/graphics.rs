//! Graphics/plotting data model.
//!
//! The runtime produces graphics state; the desktop app renders it.
//! The runtime itself has NO dependency on any GUI crate — these types are
//! plain data so they can cross the worker→UI channel unchanged.

use matlab_values::DenseMatrix;

#[derive(Debug, Clone, PartialEq)]
pub enum PlotItem {
    /// 2-D polyline: label, x-values, y-values (equal length).
    Line {
        label: String,
        x: Vec<f64>,
        y: Vec<f64>,
    },
    /// Marker plot (no connecting line).
    Scatter {
        label: String,
        x: Vec<f64>,
        y: Vec<f64>,
    },
    /// Bar chart: label, x positions, bar heights (equal length).
    Bar {
        label: String,
        positions: Vec<f64>,
        values: Vec<f64>,
    },
    /// Histogram: raw values with a bin count.
    Histogram {
        label: String,
        values: Vec<f64>,
        bins: usize,
    },
}

impl PlotItem {
    pub fn label(&self) -> &str {
        match self {
            PlotItem::Line { label, .. }
            | PlotItem::Scatter { label, .. }
            | PlotItem::Bar { label, .. }
            | PlotItem::Histogram { label, .. } => label,
        }
    }

    pub fn set_label(&mut self, label: &str) {
        match self {
            PlotItem::Line { label: l, .. }
            | PlotItem::Scatter { label: l, .. }
            | PlotItem::Bar { label: l, .. }
            | PlotItem::Histogram { label: l, .. } => *l = label.to_string(),
        }
    }
}

/// One figure window's contents.
#[derive(Debug, Clone, PartialEq)]
pub struct FigureState {
    pub handle: usize,
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub legend_shown: bool,
    pub grid: bool,
    pub items: Vec<PlotItem>,
}

impl FigureState {
    /// A fresh, empty figure with the given handle (also used by `clf`).
    pub fn fresh(handle: usize) -> Self {
        Self {
            handle,
            title: None,
            xlabel: None,
            ylabel: None,
            legend_shown: false,
            grid: false,
            items: Vec::new(),
        }
    }
}

/// Persistent graphics state across commands in a session.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GraphicsState {
    pub figures: Vec<FigureState>,
    /// Index into `figures` of the current figure (created lazily on use).
    pub current: Option<usize>,
}

impl GraphicsState {
    /// Current figure, creating one (handle = max+1) when none exists.
    pub fn current_or_new(&mut self) -> &mut FigureState {
        if self.current.is_none() || self.current.unwrap() >= self.figures.len() {
            let handle = self.figures.iter().map(|f| f.handle).max().unwrap_or(0) + 1;
            self.figures.push(FigureState::fresh(handle));
            self.current = Some(self.figures.len() - 1);
        }
        &mut self.figures[self.current.unwrap()]
    }

    /// Select figure by 1-based handle, creating it if absent (MATLAB `figure(n)` semantics).
    pub fn select_handle(&mut self, handle: usize) {
        if let Some(pos) = self.figures.iter().position(|f| f.handle == handle) {
            self.current = Some(pos);
        } else {
            self.figures.push(FigureState::fresh(handle));
            self.current = Some(self.figures.len() - 1);
        }
    }

    /// Remove the current figure (`close`); selection becomes unset.
    pub fn close_current(&mut self) {
        if let Some(pos) = self.current.take() {
            self.figures.remove(pos);
        }
    }

    /// Remove a figure by handle if present (`close(n)`).
    pub fn remove_handle(&mut self, handle: usize) {
        if let Some(pos) = self.figures.iter().position(|f| f.handle == handle) {
            self.figures.remove(pos);
            if self.current.map(|c| c == pos).unwrap_or(false) {
                self.current = None;
            }
        }
    }

    pub fn clear_all(&mut self) {
        self.figures.clear();
        self.current = None;
    }
}

/// Extract a flat value column from any 1xN / Nx1 / 1x1 matrix (for plotting).
/// Errors carry a MATLAB-style message when the shape is not vector-like.
pub fn vector_from_matrix(
    m: &DenseMatrix,
    fn_name: &str,
) -> Result<Vec<f64>, matlab_diagnostics::Diagnostic> {
    let _ = fn_name;
    if m.is_scalar() {
        return Ok(vec![m.data[0]]);
    }
    if m.rows == 1 || m.cols == 1 {
        let mut out = Vec::with_capacity(m.data.len());
        if m.rows == 1 {
            out.extend_from_slice(&m.data);
        } else {
            for r in 0..m.rows {
                out.push(m.data[r * m.cols]); // column vector: first element of each row
            }
        }
        return Ok(out);
    }
    Err(matlab_diagnostics::Diagnostic::error(
        format!(
            "{}: data must be numeric vectors (got a {}x{} matrix); 1xN or Nx1 supported.",
            fn_name, m.rows, m.cols
        ),
        None,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_extraction_shapes() {
        let row = DenseMatrix::from_rows(vec![vec![1.0, 2.0, 3.0]], None).unwrap();
        let col = DenseMatrix::from_rows(vec![vec![1.0], vec![2.0], vec![3.0]], None).unwrap();
        let scalar = DenseMatrix::scalar(7.0);
        let mat = DenseMatrix::from_rows(vec![vec![1.0, 2.0], vec![3.0, 4.0]], None).unwrap();
        assert_eq!(
            vector_from_matrix(&row, "plot").unwrap(),
            vec![1.0, 2.0, 3.0]
        );
        assert_eq!(
            vector_from_matrix(&col, "plot").unwrap(),
            vec![1.0, 2.0, 3.0]
        );
        assert_eq!(vector_from_matrix(&scalar, "plot").unwrap(), vec![7.0]);
        assert!(vector_from_matrix(&mat, "plot").is_err());
    }

    #[test]
    fn figure_lifecycle() {
        let mut g = GraphicsState::default();
        let f = g.current_or_new();
        f.items.push(PlotItem::Line {
            label: "a".into(),
            x: vec![0.0],
            y: vec![0.0],
        });
        assert_eq!(g.figures.len(), 1);
        assert_eq!(g.figures[0].handle, 1);

        g.select_handle(3);
        assert_eq!(g.figures[0].handle, 1);
        assert_eq!(g.figures.last().unwrap().handle, 3);
        assert_eq!(g.current, Some(1));

        g.clear_all();
        assert!(g.figures.is_empty());
        assert_eq!(g.current, None);
    }
}
