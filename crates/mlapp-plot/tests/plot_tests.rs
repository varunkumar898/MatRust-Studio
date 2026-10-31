use mlapp_plot::AxesModel;

#[test]
fn test_axes_model_plotting_and_svg_export() {
    let mut axes = AxesModel::new("UIAxes");
    axes.title = "Step Response".to_string();
    axes.xlabel = "Time (s)".to_string();
    axes.ylabel = "Amplitude".to_string();

    let t = vec![0.0, 0.5, 1.0, 1.5, 2.0];
    let y = vec![0.0, 0.39, 0.63, 0.77, 0.86];

    axes.plot(t, y, Some("Response".to_string()));
    assert_eq!(axes.series.len(), 1);

    let svg = axes.to_svg(400.0, 300.0);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("Step Response"));
    assert!(svg.contains("polyline"));
    assert!(svg.contains("</svg>"));
}
