
use matlabr_apps::simulation::{simulate_rk4, SystemModel, InputSignal, SimulationParams};

fn main() {
    let tau = 1.0;
    let gain = 2.0;
    let model = SystemModel::FirstOrder { tau, gain, y0: 0.0 };
    let canonical = model.to_state_space().unwrap();
    let dt = 0.001;
    let input = InputSignal::Impulse { magnitude: 1.0, t_start: 0.0 };
    let params = SimulationParams { t_final: 2.0, dt };
    let res = simulate_rk4(&canonical, &input, &params, None).unwrap();
    for i in [1, 2, 10, 100, 500, 1000] {
        let t = res.time[i];
        let y = res.y[i];
        let expected = (gain / tau) * (-t / tau).exp();
        println!("t={:.3}: y={:.4}, expected={:.4}, ratio={:.4}", t, y, expected, y / expected);
    }
}
