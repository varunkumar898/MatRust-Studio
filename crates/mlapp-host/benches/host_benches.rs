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

fn bench_pure_interpreter_loop(c: &mut Criterion) {
    let mut interp = rustlab_eval::Interpreter::new();
    let script = "acc = 0;\nfor i = 1:100000\n    acc = acc + i;\nend";

    c.bench_function("interpreter_pure_for_loop_1e5", |b| {
        b.iter(|| {
            let res = interp.eval_script(black_box(script)).expect("eval script");
            black_box(res);
        })
    });
}

fn bench_indexing_growth(c: &mut Criterion) {
    let mut interp = rustlab_eval::Interpreter::new();
    rustlab_stdlib::register_all(&mut interp);
    let script = "x = [1];\nfor i = 1:2000\n    x(end+1) = i;\nend";

    c.bench_function("interpreter_indexing_growth_2e3", |b| {
        b.iter(|| {
            let res = interp.eval_script(black_box(script)).expect("eval script");
            black_box(res);
        })
    });
}

fn bench_matrix_multiply_256x256(c: &mut Criterion) {
    let mut interp = rustlab_eval::Interpreter::new();
    rustlab_stdlib::register_all(&mut interp);
    let init_script = "A = rand(256, 256); B = rand(256, 256);";
    let _ = interp.eval_script(init_script).expect("init matrices");
    let script = "C = A * B;";

    c.bench_function("interpreter_matrix_multiply_256x256", |b| {
        b.iter(|| {
            let res = interp.eval_script(black_box(script)).expect("eval script");
            black_box(res);
        })
    });
}

fn bench_string_building(c: &mut Criterion) {
    let mut interp = rustlab_eval::Interpreter::new();
    rustlab_stdlib::register_all(&mut interp);
    let script = "s = 'a';\nfor i = 1:2000\n    s = [s, 'b'];\nend";

    c.bench_function("interpreter_string_building_2e3", |b| {
        b.iter(|| {
            let res = interp.eval_script(black_box(script)).expect("eval script");
            black_box(res);
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = bench_container_open,
        bench_ui_xml_parse,
        bench_interpreter_callback,
        bench_pure_interpreter_loop,
        bench_indexing_growth,
        bench_matrix_multiply_256x256,
        bench_string_building
);
criterion_main!(benches);
