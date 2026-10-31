use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mlapp_container::AppContainer;
use mlapp_host::{AppSession, HostConfig, UiEvent};
use mlapp_ui::parse_app_xml;
use std::io::Cursor;

fn get_sample_app_bytes() -> Vec<u8> {
    let candidate_paths = [
        "fixtures/sample_apps/r2021a_sample.mlapp",
        "crates/mlapp-container/fixtures/sample_apps/r2021a_sample.mlapp",
        "../../fixtures/sample_apps/r2021a_sample.mlapp",
    ];
    for p in candidate_paths {
        if let Ok(data) = std::fs::read(p) {
            return data;
        }
    }
    panic!("Could not locate sample app fixture for benchmarking");
}

fn bench_container_open(c: &mut Criterion) {
    let bytes = get_sample_app_bytes();
    c.bench_function("container_open_and_inspect", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(bytes.clone()));
            let mut container = AppContainer::open(cursor).expect("open container");
            let _ = black_box(container.inspect().expect("inspect package"));
            let _ = container
                .read_entry_to_string("matlab/document.xml")
                .expect("read document.xml");
        })
    });
}

fn bench_ui_xml_parse(c: &mut Criterion) {
    let bytes = get_sample_app_bytes();
    let cursor = Cursor::new(bytes);
    let mut container = AppContainer::open(cursor).expect("open container");
    let xml = container
        .read_entry_to_string("appdesigner/app.xml")
        .expect("read app.xml");

    c.bench_function("ui_xml_parse_component_tree", |b| {
        b.iter(|| {
            let tree = parse_app_xml(black_box(&xml)).expect("parse app.xml");
            black_box(tree);
        })
    });
}

fn bench_interpreter_callback(c: &mut Criterion) {
    let bytes = get_sample_app_bytes();

    c.bench_function("interpreter_session_and_button_callback", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(bytes.clone()));
            let mut session =
                AppSession::launch(cursor, HostConfig::default()).expect("launch session");
            let _ = session.send_event(UiEvent::ButtonClick {
                component_name: "StartBtn".to_string(),
            });
            std::thread::sleep(std::time::Duration::from_millis(5));
            session.shutdown();
        })
    });
}

criterion_group!(
    benches,
    bench_container_open,
    bench_ui_xml_parse,
    bench_interpreter_callback
);
criterion_main!(benches);
