//! Thread-local plot state mirroring MATLAB's `gcf`/`gca` within one
//! interpreter session. Reset between tests with [`plot_state_reset`].

use crate::PlotRegistry;
use std::cell::RefCell;

thread_local! {
    /// Holds all figures and the current figure for the current thread.
    static PLOT_STATE: RefCell<PlotRegistry> = RefCell::new(PlotRegistry::new());
}

/// Execute `f` with mutable access to the current plot registry.
#[allow(dead_code)]       // used by rustlab-stdlib builtins
#[allow(clippy::needless_borrow)] // &mut RefMut auto-derefs to &mut PlotRegistry
pub fn with_plot_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut PlotRegistry) -> R,
{
    PLOT_STATE.with(|cell| f(&mut cell.borrow_mut()))
}

/// Execute `f` with shared access to the current plot registry.
#[allow(dead_code)]       // used by rustlab-stdlib builtins
pub fn with_plot_state_ref<F, R>(f: F) -> R
where
    F: FnOnce(&PlotRegistry) -> R,
{
    PLOT_STATE.with(|cell| f(&cell.borrow()))
}

/// Reset the plot registry to an empty state (no figures). Used between
/// tests so each test starts from a clean slate.
pub fn plot_state_reset() {
    PLOT_STATE.with(|cell| {
        *cell.borrow_mut() = PlotRegistry::new();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_get_current_figure() {
        plot_state_reset();
        let id = with_plot_state(|reg| reg.create_figure());
        let cur_id = with_plot_state_ref(|reg| reg.current_id);
        assert_eq!(cur_id, Some(id));
        let title = with_plot_state_ref(|reg| reg.current_figure().unwrap().title.clone());
        assert_eq!(title, format!("Figure {}", id));
    }

    #[test]
    fn reset_clears_figures() {
        plot_state_reset();
        let id1 = with_plot_state(|reg| reg.create_figure());
        plot_state_reset();
        let id2 = with_plot_state(|reg| reg.create_figure());
        assert_eq!(id1, id2); // ids restart at 1 after reset
        assert!(with_plot_state_ref(|reg| reg.current_figure().is_some()));
    }

    #[test]
    fn multiple_figures_tracked() {
        plot_state_reset();
        let id1 = with_plot_state(|reg| reg.create_figure());
        let id2 = with_plot_state(|reg| reg.create_figure());
        assert_eq!(with_plot_state_ref(|reg| reg.figure_ids()), vec![id1, id2]);
        assert!(with_plot_state(|reg| reg.set_current(id1)));
        let cur = with_plot_state_ref(|reg| reg.current_figure().unwrap().id);
        assert_eq!(cur, id1);
    }
}
