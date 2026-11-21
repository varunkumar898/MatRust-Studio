//! Tests for MG5 2D and 3D Plotting Types

use rustlab_math::{DenseArray, Value};
use rustlab_stdlib::plot_builtins::*;

#[test]
fn test_plot_types_mesh() {
    rustlab_plot::plot_state_reset();
    figure_fn(&[], 1).unwrap();
    let z = Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 2.0, 3.0, 4.0]).unwrap());
    mesh_fn(&[z], 0).unwrap();

    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.series.len(), 1);
    assert!(matches!(ax.series[0], rustlab_plot::Series::Surface(_)));
    assert!(ax.colorbar.is_some());
}

#[test]
fn test_plot_types_stem() {
    rustlab_plot::plot_state_reset();
    figure_fn(&[], 1).unwrap();
    let y = Value::Double(DenseArray::from_vec(vec![1, 3], vec![2.0, 4.0, 1.0]).unwrap());
    stem_fn(&[y], 0).unwrap();

    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert!(!ax.series.is_empty());
    assert!(matches!(ax.series[0], rustlab_plot::Series::Line(_)));
}

#[test]
fn test_plot_types_log_scales() {
    rustlab_plot::plot_state_reset();
    figure_fn(&[], 1).unwrap();
    let x = Value::Double(DenseArray::from_vec(vec![1, 3], vec![1.0, 10.0, 100.0]).unwrap());
    let y = Value::Double(DenseArray::from_vec(vec![1, 3], vec![2.0, 3.0, 5.0]).unwrap());

    // semilogx
    semilogx_fn(&[x.clone(), y.clone()], 0).unwrap();
    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.xscale, rustlab_plot::Scale::Log);
    assert_eq!(ax.yscale, rustlab_plot::Scale::Linear);

    // semilogy
    semilogy_fn(&[x.clone(), y.clone()], 0).unwrap();
    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.yscale, rustlab_plot::Scale::Log);

    // loglog
    loglog_fn(&[x, y], 0).unwrap();
    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.xscale, rustlab_plot::Scale::Log);
    assert_eq!(ax.yscale, rustlab_plot::Scale::Log);
}

#[test]
fn test_plot_types_polar() {
    rustlab_plot::plot_state_reset();
    figure_fn(&[], 1).unwrap();
    let theta = Value::Double(
        DenseArray::from_vec(vec![1, 2], vec![0.0, std::f64::consts::PI / 2.0]).unwrap(),
    );
    let rho = Value::Double(DenseArray::from_vec(vec![1, 2], vec![1.0, 2.0]).unwrap());
    polar_fn(&[theta, rho], 0).unwrap();

    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.series.len(), 1);
    if let rustlab_plot::Series::Line(line) = &ax.series[0] {
        assert_eq!(line.x.len(), 2);
        assert!((line.x[0] - 1.0).abs() < 1e-10);
        assert!(line.x[1].abs() < 1e-10);
        assert!(line.y[0].abs() < 1e-10);
        assert!((line.y[1] - 2.0).abs() < 1e-10);
    } else {
        panic!("Expected Line series for polar plot");
    }
}

#[test]
fn test_plot_types_quiver() {
    rustlab_plot::plot_state_reset();
    figure_fn(&[], 1).unwrap();
    let u = Value::Double(DenseArray::from_vec(vec![1, 2], vec![1.0, 0.0]).unwrap());
    let v = Value::Double(DenseArray::from_vec(vec![1, 2], vec![0.0, 1.0]).unwrap());
    quiver_fn(&[u, v], 0).unwrap();

    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert!(!ax.series.is_empty());
    assert!(matches!(ax.series[0], rustlab_plot::Series::Line(_)));
}

#[test]
fn test_plot_types_contour_and_imagesc() {
    rustlab_plot::plot_state_reset();
    figure_fn(&[], 1).unwrap();
    let z = Value::Double(DenseArray::from_vec(vec![2, 2], vec![1.0, 2.0, 3.0, 4.0]).unwrap());

    // contour
    contour_fn(std::slice::from_ref(&z), 0).unwrap();
    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.camera.azimuth, 0.0);
    assert_eq!(ax.camera.elevation, 90.0);

    // imagesc
    imagesc_fn(std::slice::from_ref(&z), 0).unwrap();
    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert!(ax.colorbar.is_some());
    assert_eq!(ax.camera.elevation, 90.0);

    // imagesc with (X, Y, C) regression test
    let x = Value::Double(DenseArray::from_vec(vec![1, 2], vec![1.0, 2.0]).unwrap());
    let y = Value::Double(DenseArray::from_vec(vec![1, 2], vec![10.0, 20.0]).unwrap());
    imagesc_fn(&[x, y, z], 0).unwrap();
}

#[test]
fn test_plot_types_area_and_fill() {
    rustlab_plot::plot_state_reset();
    figure_fn(&[], 1).unwrap();
    let y = Value::Double(DenseArray::from_vec(vec![1, 3], vec![1.0, 3.0, 2.0]).unwrap());
    area_fn(&[y], 0).unwrap();

    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.series.len(), 1);

    let x = Value::Double(DenseArray::from_vec(vec![1, 3], vec![0.0, 1.0, 0.5]).unwrap());
    let y = Value::Double(DenseArray::from_vec(vec![1, 3], vec![0.0, 0.0, 1.0]).unwrap());
    fill_fn(&[x, y], 0).unwrap();
}

#[test]
fn test_plot_types_view_and_axis() {
    rustlab_plot::plot_state_reset();
    figure_fn(&[], 1).unwrap();

    view_fn(&[Value::scalar(45.0), Value::scalar(30.0)], 0).unwrap();
    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.camera.azimuth, 45.0);
    assert_eq!(ax.camera.elevation, 30.0);

    let lims = Value::Double(DenseArray::from_vec(vec![1, 4], vec![0.0, 10.0, -5.0, 5.0]).unwrap());
    axis_fn(&[lims], 0).unwrap();
    let fig = rustlab_plot::with_plot_state_ref(|reg| reg.current_figure().unwrap().clone());
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.xlim, Some([0.0, 10.0]));
    assert_eq!(ax.ylim, Some([-5.0, 5.0]));
}
