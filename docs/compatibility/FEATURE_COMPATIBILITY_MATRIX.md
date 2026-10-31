# MatRust Studio Feature Compatibility Matrix

## 1. Feature Status Overview

| Domain | Backend Source Files | Command Support | GUI Integration | Automated Tests | Status |
|---|---|---|---|---|---|
| **Mathematics & Calculations** | `crates/rustlab-math`, `crates/rustlab-stdlib/src/math_elem.rs`, `numerical.rs` | `+`, `-`, `*`, `/`, `^`, `roots`, `polyval`, `polyfit`, `trapz`, `cumtrapz`, `gradient`, `interp1`, elementary functions | Interactive Command Window, Editor script execution | `array_tests.rs`, `numerical_tests.rs`, `stdlib_tests.rs` (38 tests) | **Complete** |
| **Matrix Operations & Linear Algebra** | `crates/rustlab-stdlib/src/linalg/`, `crates/rustlab-math` | `\`, `/`, `inv`, `det`, `rank`, `trace`, `norm`, `lu`, `qr`, `chol`, `eig`, `svd`, `pinv`, `cond`, `null`, `orth`, `expm`, `kron` | Workspace browser, Command Window | `stdlib_a2_tests.rs`, `stdlib_tests.rs`, `desktop_tests.rs` (35 tests) | **Complete** |
| **Data Analysis & Statistics** | `crates/rustlab-data/src/stats.rs`, `tables.rs` | `mean`, `median`, `mode`, `var`, `std`, `prctile`, `quantile`, `cov`, `corrcoef`, `movmean`, `zscore`, `head`, `tail`, `summary` | Workspace browser, variable inspector | `data_tests.rs`, `table_tests.rs`, `g1_stats_reference_tests.rs` (47 tests) | **Complete** |
| **Import / Export** | `crates/rustlab-data/src/csv_io.rs`, `tables.rs`, `crates/rustlab-matio` | `readmatrix`, `writematrix`, `readtable`, `writetable`, `load`, `save` | Dedicated Import Data dialog, File explorer panel | `data_tests.rs`, `table_tests.rs`, `matio_tests.rs`, `desktop_tests.rs` (18 tests) | **Complete** |
| **Figures & Visualization** | `apps/desktop/src/figures.rs`, `execution.rs` | `plot`, `scatter`, `bar`, `title`, `xlabel`, `ylabel`, `grid`, `figure`, `clf`, `close` | Dedicated Figures dockable panel with series selection & canvas | `desktop_tests.rs` (`test_session_plotting_builtins_and_state`) | **Complete** |
| **Engineering Simulation** | `crates/rustlab-stdlib/src/simulation.rs` | `tf`, `step` (continuous-time 1st and 2nd order dynamic response) | Scope visualization in Figures panel | `extended_capabilities_tests.rs`, `desktop_tests.rs` | **Complete** (1st/2nd order analytical & state-space); **Partial** (visual graphical block diagram canvas is Planned) |
| **Signal Processing** | `crates/rustlab-stdlib/src/signal.rs`, `signal_proc.rs` | `fft`, `ifft`, `fft2`, `ifft2`, `fftshift`, `filter`, `filtfilt`, `hamming`, `hann`, `blackman`, `xcorr` | Real-time plotting in Figures panel | `stdlib_a2_tests.rs`, `extended_capabilities_tests.rs` (8 tests) | **Complete** |
| **Image Processing** | `crates/rustlab-stdlib/src/image_proc.rs` | `rgb2gray`, `imresize`, `imfilter` | 2D matrix inspection, image array display | `extended_capabilities_tests.rs` | **Complete** (grayscale conversion, bilinear resizing, spatial correlation filtering) |
| **Machine Learning** | `crates/rustlab-stdlib/src/ml.rs` | `fitlm`, `predict`, `kmeans`, `confusionmat` | Workspace model inspection | `extended_capabilities_tests.rs` (2 tests) | **Complete** (classical linear regression, k-means clustering, confusion matrix) |
| **Robotics & Kinematics** | `crates/rustlab-stdlib/src/robotics.rs` | `rotx`, `roty`, `rotz`, `trotx`, `troty`, `trotz`, `transl`, `fkine_2link`, `ikine_2link`, `jtraj` | Coordinate frame and trajectory display | `extended_capabilities_tests.rs` (3 tests) | **Complete** (3D rotations, homogeneous transforms, planar arm kinematics, quintic trajectories) |
| **Programming & Algorithm Development** | `crates/rustlab-parse`, `crates/rustlab-eval`, `apps/desktop/src/editor.rs` | `.m` scripts, local functions, closures, for/while loops, if/else, breakpoints, step/continue | Multi-tab editor with code folding, gutter breakpoints, syntax highlighting | `parse_tests.rs`, `eval_tests.rs`, `desktop_tests.rs` (67 tests) | **Complete** |
| **App Designer** | `apps/desktop/src/dock.rs`, `app.rs` | Multi-panel docking, custom layouts, drag-and-drop tab manipulation | Native IDE docking system | `desktop_tests.rs` | **Partial** (IDE panel docking and customization complete; end-user visual widget drag-and-drop canvas is Planned) |
| **Embedded Systems & Hardware** | `crates/rustlab-stdlib`, hardware trait mocks | Hardware data streaming mocks, C99 code generation architecture | Serial connection management interface | Code generation test harness | **Planned** / **Partial** (trait abstractions and C99 generator exist; live USB serial communication is Blocked pending physical target attachment) |

## 2. Detailed Verification Evidence per Domain

### Mathematics & Calculations (Complete)
- **Source**: `crates/rustlab-stdlib/src/numerical.rs`, `crates/rustlab-stdlib/src/math_elem.rs`
- **APIs**: `roots_fn`, `polyval_fn`, `polyfit_fn`, `trapz_fn`, `cumtrapz_fn`, `gradient_fn`, `interp1_fn`
- **Automated Tests**: `crates/rustlab-stdlib/tests/numerical_tests.rs` (6 tests passing)

### Data Analysis & Tables (Complete)
- **Source**: `crates/rustlab-data/src/tables.rs`, `crates/rustlab-data/src/stats.rs`
- **APIs**: `readtable`, `writetable`, `head_fn`, `tail_fn`, `summary_fn`
- **Automated Tests**: `crates/rustlab-stdlib/tests/table_tests.rs` (3 tests passing)

### Engineering Simulation (Complete Backend, Partial Visual Block Canvas)
- **Source**: `crates/rustlab-stdlib/src/simulation.rs`
- **APIs**: `tf_fn`, `step_fn`
- **Automated Tests**: `crates/rustlab-stdlib/tests/extended_capabilities_tests.rs` (`test_simulation_tf_and_step`), `apps/desktop/tests/desktop_tests.rs` (`test_desktop_session_simulation_and_robotics`)

### Signal & Image Processing (Complete)
- **Source**: `crates/rustlab-stdlib/src/signal_proc.rs`, `image_proc.rs`
- **APIs**: `filter_fn`, `filtfilt_fn`, `hamming_fn`, `hann_fn`, `blackman_fn`, `xcorr_fn`, `rgb2gray_fn`, `imresize_fn`, `imfilter_fn`
- **Automated Tests**: `crates/rustlab-stdlib/tests/extended_capabilities_tests.rs` (2 tests passing)

### Machine Learning (Complete)
- **Source**: `crates/rustlab-stdlib/src/ml.rs`
- **APIs**: `fitlm_fn`, `predict_fn`, `kmeans_fn`, `confusionmat_fn`
- **Automated Tests**: `crates/rustlab-stdlib/tests/extended_capabilities_tests.rs` (`test_machine_learning_fitlm_predict_kmeans`)

### Robotics (Complete)
- **Source**: `crates/rustlab-stdlib/src/robotics.rs`
- **APIs**: `rotx_fn`, `roty_fn`, `rotz_fn`, `trotx_fn`, `troty_fn`, `trotz_fn`, `transl_fn`, `fkine_2link_fn`, `ikine_2link_fn`, `jtraj_fn`
- **Automated Tests**: `crates/rustlab-stdlib/tests/extended_capabilities_tests.rs` (`test_robotics_rotations_and_kinematics`), `apps/desktop/tests/desktop_tests.rs`
