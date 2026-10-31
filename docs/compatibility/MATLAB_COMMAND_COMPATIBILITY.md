# MATLAB Command Compatibility Registry

## 1. Registry Table

This registry documents the mapping between MATLAB command syntax, the underlying Rust backend implementation, and verification status.

| MATLAB Command | Backend Crate / Module | Backend Implementation Function | Verified in Automated Tests | Status |
|---|---|---|---|---|
| `+`, `-`, `*`, `.*`, `/`, `./`, `^`, `.^` | `rustlab-math::ops` | `elementwise_binary`, `matmul_2d` | Yes (`array_tests.rs`, `eval_tests.rs`) | Complete |
| `det` | `rustlab-stdlib::linalg` | `det_fn` | Yes (`stdlib_tests.rs`) | Complete |
| `inv` | `rustlab-stdlib::linalg` | `inv_fn` | Yes (`stdlib_tests.rs`, `desktop_tests.rs`) | Complete |
| `rank` | `rustlab-stdlib::linalg` | `rank_fn` | Yes (`stdlib_a2_tests.rs`) | Complete |
| `trace` | `rustlab-stdlib::linalg` | `trace_fn` | Yes (`stdlib_tests.rs`) | Complete |
| `norm` | `rustlab-stdlib::linalg` | `norm_fn` | Yes (`stdlib_tests.rs`) | Complete |
| `mldivide` (`\`) | `rustlab-stdlib::linalg` | `mldivide_fn` | Yes (`stdlib_tests.rs`, `desktop_tests.rs`) | Complete |
| `lu` | `rustlab-stdlib::linalg` | `lu_fn` | Yes (`stdlib_tests.rs`) | Complete |
| `qr` | `rustlab-stdlib::linalg` | `qr_fn`, `qr_pivoted_fn` | Yes (`stdlib_a2_tests.rs`) | Complete |
| `chol` | `rustlab-stdlib::linalg` | `chol_fn` | Yes (`stdlib_tests.rs`) | Complete |
| `eig` | `rustlab-stdlib::linalg` | `eig_fn` | Yes (`stdlib_a2_tests.rs`) | Complete |
| `svd` | `rustlab-stdlib::linalg` | `svd_fn` | Yes (`stdlib_a2_tests.rs`) | Complete |
| `roots` | `rustlab-stdlib::numerical` | `roots_fn` | Yes (`numerical_tests.rs`) | Complete |
| `polyval` | `rustlab-stdlib::numerical` | `polyval_fn` | Yes (`numerical_tests.rs`) | Complete |
| `polyfit` | `rustlab-stdlib::numerical` | `polyfit_fn` | Yes (`numerical_tests.rs`) | Complete |
| `trapz` | `rustlab-stdlib::numerical` | `trapz_fn` | Yes (`numerical_tests.rs`) | Complete |
| `cumtrapz` | `rustlab-stdlib::numerical` | `cumtrapz_fn` | Yes (`numerical_tests.rs`) | Complete |
| `diff` | `rustlab-stdlib::stats` | `diff_fn` | Yes (`stdlib_tests.rs`) | Complete |
| `gradient` | `rustlab-stdlib::numerical` | `gradient_fn` | Yes (`numerical_tests.rs`) | Complete |
| `interp1` | `rustlab-stdlib::numerical` | `interp1_fn` | Yes (`numerical_tests.rs`) | Complete |
| `mean` | `rustlab-stdlib::stats` | `mean_fn` | Yes (`stdlib_tests.rs`, `data_tests.rs`) | Complete |
| `median` | `rustlab-data::stats` | `median` | Yes (`data_tests.rs`) | Complete |
| `std` | `rustlab-data::stats` | `std_fn` | Yes (`data_tests.rs`) | Complete |
| `var` | `rustlab-data::stats` | `var_fn` | Yes (`data_tests.rs`) | Complete |
| `cov` | `rustlab-data::stats` | `cov` | Yes (`data_tests.rs`) | Complete |
| `corrcoef` | `rustlab-data::stats` | `corrcoef` | Yes (`data_tests.rs`) | Complete |
| `readmatrix` | `rustlab-data::csv_io` | `readmatrix` | Yes (`data_tests.rs`) | Complete |
| `writematrix` | `rustlab-data::csv_io` | `writematrix` | Yes (`data_tests.rs`) | Complete |
| `readtable` | `rustlab-data::tables` | `readtable` | Yes (`table_tests.rs`) | Complete |
| `writetable` | `rustlab-data::tables` | `writetable` | Yes (`table_tests.rs`) | Complete |
| `head` | `rustlab-data::tables` | `head_fn` | Yes (`table_tests.rs`) | Complete |
| `tail` | `rustlab-data::tables` | `tail_fn` | Yes (`table_tests.rs`) | Complete |
| `summary` | `rustlab-data::tables` | `summary_fn` | Yes (`table_tests.rs`) | Complete |
| `load` | `rustlab-stdlib::matio_builtins` | `load` | Yes (`matio_tests.rs`) | Complete |
| `save` | `rustlab-stdlib::matio_builtins` | `save` | Yes (`matio_tests.rs`) | Complete |
| `tf` | `rustlab-stdlib::simulation` | `tf_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `step` | `rustlab-stdlib::simulation` | `step_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `filter` | `rustlab-stdlib::signal_proc` | `filter_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `filtfilt` | `rustlab-stdlib::signal_proc` | `filtfilt_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `hamming` | `rustlab-stdlib::signal_proc` | `hamming_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `hann` | `rustlab-stdlib::signal_proc` | `hann_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `blackman` | `rustlab-stdlib::signal_proc` | `blackman_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `xcorr` | `rustlab-stdlib::signal_proc` | `xcorr_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `rgb2gray` | `rustlab-stdlib::image_proc` | `rgb2gray_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `imresize` | `rustlab-stdlib::image_proc` | `imresize_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `imfilter` | `rustlab-stdlib::image_proc` | `imfilter_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `fitlm` | `rustlab-stdlib::ml` | `fitlm_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `predict` | `rustlab-stdlib::ml` | `predict_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `kmeans` | `rustlab-stdlib::ml` | `kmeans_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `confusionmat` | `rustlab-stdlib::ml` | `confusionmat_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `rotx`, `roty`, `rotz` | `rustlab-stdlib::robotics` | `rotx_fn`, `roty_fn`, `rotz_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `trotx`, `troty`, `trotz` | `rustlab-stdlib::robotics` | `trotx_fn`, `troty_fn`, `trotz_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `transl` | `rustlab-stdlib::robotics` | `transl_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `fkine_2link` | `rustlab-stdlib::robotics` | `fkine_2link_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `ikine_2link` | `rustlab-stdlib::robotics` | `ikine_2link_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
| `jtraj` | `rustlab-stdlib::robotics` | `jtraj_fn` | Yes (`extended_capabilities_tests.rs`) | Complete |
