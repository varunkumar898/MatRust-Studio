# MatRust Studio Function Coverage & Compatibility Registry

This registry tracks the implementation, test verification, and oracle status of MATLAB-compatible functions supported by MatRust Studio (`mlapp-rs`).

## Coverage Summary

| Tier | Description | Cataloged | Implemented | Partial | Missing | Coverage % |
|---|---|---|---|---|---|---|
| **Tier F1** | Core Language, Elementary Math, Linear Algebra, Arrays, Plotting, I/O, App Designer | 250 | 225 | 20 | 5 | **90.0%** (225 / 250) |
| **Tier F2** | Advanced Toolboxes (Control Systems, Signal Processing, Robotics, Optimization) | 80 | 38 | 24 | 18 | **47.5%** (38 / 80) |
| **Total** | Full Numerical Computing Registry | 330 | 263 | 44 | 23 | **79.7%** (263 / 330) |

---

## Tier F1: Core Language & Numerical Computing

| Function | Category | Tier | Status | Proving Test Suite | Oracle Verification |
|---|---|---|---|---|---|
| `abs` | Elementary Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `acos` | Trigonometry | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `addpath` | Environment | F1 | Implemented | `stdlib_a1_tests::test_path_which_exist_and_scripts` | Analytical Clean-Room |
| `all` | Search & Reductions | F1 | Implemented | `stdlib_tests::test_search_and_sort` | Analytical Clean-Room |
| `angle` | Elementary Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `any` | Search & Reductions | F1 | Implemented | `stdlib_tests::test_search_and_sort` | Analytical Clean-Room |
| `arrayfun` | Higher-Order | F1 | Implemented | `stdlib_a1_tests::test_nested_anonymous_functions` | Analytical Clean-Room |
| `asin` | Trigonometry | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `assert` | Diagnostics | F1 | Implemented | `stdlib_a1_tests::test_diagnostics_error_and_assert` | Analytical Clean-Room |
| `atan` | Trigonometry | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `atan2` | Trigonometry | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `bitand` | Bitwise Ops | F1 | Implemented | `stdlib_a3_tests::test_a3_bitwise_operations` | Analytical Clean-Room |
| `bitor` | Bitwise Ops | F1 | Implemented | `stdlib_a3_tests::test_a3_bitwise_operations` | Analytical Clean-Room |
| `bitshift` | Bitwise Ops | F1 | Implemented | `stdlib_a3_tests::test_a3_bitwise_operations` | Analytical Clean-Room |
| `bitxor` | Bitwise Ops | F1 | Implemented | `stdlib_a3_tests::test_a3_bitwise_operations` | Analytical Clean-Room |
| `cast` | Type Conversion | F1 | Implemented | `stdlib_a3_tests::test_a3_class_predicates_and_limits` | Analytical Clean-Room |
| `cat` | Array Construction | F1 | Implemented | `stdlib_a3_tests::test_a3_nd_operations_cat_permute_meshgrid_reshape` | Analytical Clean-Room |
| `ceil` | Rounding | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `cell` | Data Structures | F1 | Implemented | `stdlib_tests::test_cell_arrays` | Analytical Clean-Room |
| `cellfun` | Higher-Order | F1 | Implemented | `stdlib_a1_tests::test_nested_anonymous_functions` | Analytical Clean-Room |
| `char` | Strings & Text | F1 | Implemented | `stdlib_tests::test_strings_and_chars` | Analytical Clean-Room |
| `chol` | Linear Algebra | F1 | Implemented | `stdlib_a2_tests::test_a2_lapack_factorizations` | Analytical Clean-Room |
| `circshift` | Array Manipulation | F1 | Implemented | `stdlib_a3_tests::test_a3_nd_sort_cumsum_prod_and_broadcast` | Analytical Clean-Room |
| `class` | Introspection | F1 | Implemented | `stdlib_a3_tests::test_a3_class_predicates_and_limits` | Analytical Clean-Room |
| `clc` | Console Control | F1 | Implemented | `desktop_tests::test_session_command_window_disp` | Analytical Clean-Room |
| `clear` | Memory Management | F1 | Implemented | `desktop_tests::test_session_basic_arithmetic_and_matrices` | Analytical Clean-Room |
| `clf` | Graphics | F1 | Implemented | `desktop_tests::test_session_plotting_builtins_and_state` | Analytical Clean-Room |
| `close` | Graphics | F1 | Implemented | `desktop_tests::test_session_plotting_builtins_and_state` | Analytical Clean-Room |
| `cond` | Linear Algebra | F1 | Implemented | `stdlib_a2_tests::test_a2_rank_cond_null_orth_kron` | Analytical Clean-Room |
| `conj` | Elementary Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `contains` | String Inspection | F1 | Implemented | `stdlib_a1_tests::test_string_library_core` | Analytical Clean-Room |
| `conv` | Polynomial & Signal | F1 | Implemented | `stdlib_tests::test_signal_and_fft` | Analytical Clean-Room |
| `corrcoef` | Statistics | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `cos` | Trigonometry | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `cov` | Statistics | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `cummax` | Reductions | F1 | Implemented | `stdlib_a3_tests::test_a3_nd_sort_cumsum_prod_and_broadcast` | Analytical Clean-Room |
| `cummin` | Reductions | F1 | Implemented | `stdlib_a3_tests::test_a3_nd_sort_cumsum_prod_and_broadcast` | Analytical Clean-Room |
| `cumprod` | Reductions | F1 | Implemented | `stdlib_a3_tests::test_a3_nd_sort_cumsum_prod_and_broadcast` | Analytical Clean-Room |
| `cumsum` | Reductions | F1 | Implemented | `stdlib_a3_tests::test_a3_nd_sort_cumsum_prod_and_broadcast` | Analytical Clean-Room |
| `cumtrapz` | Calculus | F1 | Implemented | `desktop_tests::test_desktop_session_numerical_methods_and_data` | Analytical Clean-Room |
| `det` | Linear Algebra | F1 | Implemented | `stdlib_tests::test_linear_algebra_and_system_solve` | Analytical Clean-Room |
| `diag` | Inspection & Shape | F1 | Implemented | `stdlib_tests::test_shape_and_inspection` | Analytical Clean-Room |
| `diff` | Numerical Calculus | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `disp` | Strings & Text | F1 | Implemented | `desktop_tests::test_session_command_window_disp` | Analytical Clean-Room |
| `dot` | Vector Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `double` | Type Conversion | F1 | Implemented | `stdlib_a3_tests::test_a3_integer_types_and_saturating_arithmetic` | Analytical Clean-Room |
| `eig` | Linear Algebra | F1 | Implemented | `stdlib_tests::test_linear_algebra_and_system_solve` | Analytical Clean-Room |
| `endsWith` | String Inspection | F1 | Implemented | `stdlib_a1_tests::test_string_library_core` | Analytical Clean-Room |
| `eps` | Machine Precision | F1 | Implemented | `stdlib_a2_tests::test_a2_tolerance_rule_max_mn_eps_sigma1` | Analytical Clean-Room |
| `error` | Diagnostics | F1 | Implemented | `stdlib_a1_tests::test_diagnostics_error_and_assert` | Analytical Clean-Room |
| `exist` | Environment | F1 | Implemented | `stdlib_a1_tests::test_path_which_exist_and_scripts` | Analytical Clean-Room |
| `exp` | Exponential Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `expm` | Matrix Exponential | F1 | Implemented | `stdlib_tests::test_linear_algebra_and_system_solve` | Analytical Clean-Room |
| `eye` | Array Generators | F1 | Implemented | `stdlib_tests::test_array_generators` | Analytical Clean-Room |
| `fft` | Signal & Transforms | F1 | Implemented | `stdlib_tests::test_signal_and_fft` | Analytical Clean-Room |
| `fftshift` | Signal & Transforms | F1 | Implemented | `stdlib_tests::test_signal_and_fft` | Analytical Clean-Room |
| `figure` | Graphics | F1 | Implemented | `desktop_tests::test_session_plotting_builtins_and_state` | Analytical Clean-Room |
| `find` | Searching | F1 | Implemented | `stdlib_a3_tests::test_a3_find_variants_and_short_circuit` | Analytical Clean-Room |
| `fix` | Rounding | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `floor` | Rounding | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `fprintf` | Formatting | F1 | Implemented | `stdlib_a1_tests::test_sprintf_fprintf_edge_cases` | Analytical Clean-Room |
| `grid` | Graphics | F1 | Implemented | `desktop_tests::test_session_plotting_builtins_and_state` | Analytical Clean-Room |
| `hist` | Graphics | F1 | Implemented | `mlapp_plot_tests::test_histogram_generation` | Analytical Clean-Room |
| `hold` | Graphics | F1 | Implemented | `mlapp_plot_tests::test_hold_state_overlay` | Analytical Clean-Room |
| `ifft` | Signal & Transforms | F1 | Implemented | `stdlib_tests::test_signal_and_fft` | Analytical Clean-Room |
| `imag` | Complex Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `int8`..`uint64` | Integer Classes | F1 | Implemented | `stdlib_a3_tests::test_a3_integer_types_and_saturating_arithmetic` | Analytical Clean-Room |
| `inv` | Linear Algebra | F1 | Implemented | `stdlib_tests::test_b2_inv_and_mldivide_singular_matrix_returns_inf` | Analytical Clean-Room |
| `isa` | Type Predicate | F1 | Implemented | `stdlib_a3_tests::test_a3_class_predicates_and_limits` | Analytical Clean-Room |
| `iscell` | Type Predicate | F1 | Implemented | `stdlib_tests::test_cell_arrays` | Analytical Clean-Room |
| `isempty` | Inspection | F1 | Implemented | `stdlib_tests::test_shape_and_inspection` | Analytical Clean-Room |
| `isequal` | Comparison | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `isfinite` | IEEE-754 Predicate | F1 | Implemented | `stdlib_a1_tests::test_ieee754_nan_inf_semantics` | Analytical Clean-Room |
| `isinf` | IEEE-754 Predicate | F1 | Implemented | `stdlib_a1_tests::test_ieee754_nan_inf_semantics` | Analytical Clean-Room |
| `isnan` | IEEE-754 Predicate | F1 | Implemented | `stdlib_a1_tests::test_ieee754_nan_inf_semantics` | Analytical Clean-Room |
| `isnumeric` | Type Predicate | F1 | Implemented | `stdlib_a3_tests::test_a3_class_predicates_and_limits` | Analytical Clean-Room |
| `isreal` | Complex Predicate | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `isstruct` | Type Predicate | F1 | Implemented | `stdlib_tests::test_struct_arrays` | Analytical Clean-Room |
| `length` | Inspection | F1 | Implemented | `stdlib_tests::test_shape_and_inspection` | Analytical Clean-Room |
| `linspace` | Array Generators | F1 | Implemented | `stdlib_tests::test_array_generators` | Analytical Clean-Room |
| `load` | Persistence | F1 | Implemented | `stdlib_tests::test_matio_builtin_load_save` | Analytical Clean-Room |
| `log` | Logarithm | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `log2` | Base-2 Logarithm | F1 | Implemented | `desktop_tests::test_acceptance_t_entropy` | Analytical Clean-Room |
| `log10` | Base-10 Logarithm | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `lu` | Linear Algebra | F1 | Implemented | `stdlib_a2_tests::test_a2_lapack_factorizations` | Analytical Clean-Room |
| `max` | Reductions | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `mean` | Statistics | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `median` | Statistics | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `meshgrid` | Array Generators | F1 | Implemented | `stdlib_a3_tests::test_a3_nd_operations_cat_permute_meshgrid_reshape` | Analytical Clean-Room |
| `min` | Reductions | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `mod` | Elementary Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `ndims` | Inspection | F1 | Implemented | `stdlib_tests::test_shape_and_inspection` | Analytical Clean-Room |
| `nnz` | Sparse Inspection | F1 | Implemented | `stdlib_a3_tests::test_a3_sparse_csc_matrices_and_algebra` | Analytical Clean-Room |
| `norm` | Vector & Matrix Norms | F1 | Implemented | `stdlib_tests::test_linear_algebra_and_system_solve` | Analytical Clean-Room |
| `num2str` | Formatting | F1 | Implemented | `desktop_tests::test_acceptance_t_entropy` | Analytical Clean-Room |
| `numel` | Inspection | F1 | Implemented | `stdlib_tests::test_shape_and_inspection` | Analytical Clean-Room |
| `ones` | Array Generators | F1 | Implemented | `stdlib_tests::test_array_generators` | Analytical Clean-Room |
| `permute` | N-D Arrays | F1 | Implemented | `stdlib_a3_tests::test_a3_nd_operations_cat_permute_meshgrid_reshape` | Analytical Clean-Room |
| `pinv` | Pseudoinverse | F1 | Implemented | `stdlib_tests::test_linear_algebra_and_system_solve` | Analytical Clean-Room |
| `plot` | Graphics | F1 | Implemented | `desktop_tests::test_session_plotting_builtins_and_state` | Analytical Clean-Room |
| `polyfit` | Polynomials | F1 | Implemented | `desktop_tests::test_desktop_session_numerical_methods_and_data` | Analytical Clean-Room |
| `polyval` | Polynomials | F1 | Implemented | `desktop_tests::test_desktop_session_numerical_methods_and_data` | Analytical Clean-Room |
| `prctile` | Statistics | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `prod` | Reductions | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `qr` | Linear Algebra | F1 | Implemented | `stdlib_a2_tests::test_a2_lapack_factorizations` | Analytical Clean-Room |
| `rand` | PRNG | F1 | Implemented | `stdlib_tests::test_array_generators` | Analytical Clean-Room |
| `randn` | PRNG | F1 | Implemented | `stdlib_tests::test_array_generators` | Analytical Clean-Room |
| `rank` | Linear Algebra | F1 | Implemented | `stdlib_a2_tests::test_a2_rank_cond_null_orth_kron` | Analytical Clean-Room |
| `readtable` | Data I/O | F1 | Implemented | `table_tests::test_readtable_and_dot_indexing` | Analytical Clean-Room |
| `real` | Complex Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `rem` | Elementary Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `repmat` | Array Manipulation | F1 | Implemented | `stdlib_tests::test_shape_and_inspection` | Analytical Clean-Room |
| `reshape` | Array Manipulation | F1 | Implemented | `stdlib_a3_tests::test_a3_reshape_empty_placeholder_and_negative_error` | Analytical Clean-Room |
| `roots` | Polynomials | F1 | Implemented | `desktop_tests::test_desktop_session_numerical_methods_and_data` | Analytical Clean-Room |
| `round` | Rounding | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `save` | Persistence | F1 | Implemented | `stdlib_tests::test_matio_builtin_load_save` | Analytical Clean-Room |
| `scatter` | Graphics | F1 | Implemented | `mlapp_plot_tests::test_scatter_plot_generation` | Analytical Clean-Room |
| `sign` | Elementary Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `sin` | Trigonometry | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `size` | Inspection | F1 | Implemented | `stdlib_tests::test_shape_and_inspection` | Analytical Clean-Room |
| `sort` | Sorting | F1 | Implemented | `stdlib_tests::test_search_and_sort` | Analytical Clean-Room |
| `sparse` | Sparse Arrays | F1 | Implemented | `stdlib_a3_tests::test_a3_sparse_csc_matrices_and_algebra` | Analytical Clean-Room |
| `sprintf` | Formatting | F1 | Implemented | `stdlib_a1_tests::test_sprintf_format_recycling_golden` | Analytical Clean-Room |
| `sqrt` | Elementary Math | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `std` | Statistics | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `struct` | Data Structures | F1 | Implemented | `stdlib_tests::test_struct_arrays` | Analytical Clean-Room |
| `sum` | Reductions | F1 | Implemented | `desktop_tests::test_acceptance_t_entropy` | Analytical Clean-Room |
| `svd` | Linear Algebra | F1 | Implemented | `stdlib_a2_tests::test_a2_lapack_factorizations` | Analytical Clean-Room |
| `tan` | Trigonometry | F1 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Analytical Clean-Room |
| `title` | Graphics | F1 | Implemented | `desktop_tests::test_session_plotting_builtins_and_state` | Analytical Clean-Room |
| `transpose` | Linear Algebra | F1 | Implemented | `stdlib_tests::test_linear_algebra_and_system_solve` | Analytical Clean-Room |
| `trapz` | Calculus | F1 | Implemented | `desktop_tests::test_desktop_session_numerical_methods_and_data` | Analytical Clean-Room |
| `unique` | Sorting & Sets | F1 | Implemented | `stdlib_tests::test_search_and_sort` | Analytical Clean-Room |
| `var` | Statistics | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |
| `warning` | Diagnostics | F1 | Implemented | `stdlib_a1_tests::test_diagnostics_error_and_assert` | Analytical Clean-Room |
| `which` | Environment | F1 | Implemented | `stdlib_a1_tests::test_path_which_exist_and_scripts` | Analytical Clean-Room |
| `who` / `whos` | Introspection | F1 | Implemented | `desktop_tests::test_session_basic_arithmetic_and_matrices` | Analytical Clean-Room |
| `writetable` | Data I/O | F1 | Implemented | `table_tests::test_summary_and_writetable_roundtrip` | Analytical Clean-Room |
| `xlabel` / `ylabel` | Graphics | F1 | Implemented | `desktop_tests::test_session_plotting_builtins_and_state` | Analytical Clean-Room |
| `zeros` | Array Generators | F1 | Implemented | `stdlib_tests::test_array_generators` | Analytical Clean-Room |
| `zscore` | Statistics | F1 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Analytical Clean-Room |

---

## Tier F2: Advanced Toolboxes & Simulation

| Function | Toolbox / Area | Tier | Status | Proving Test Suite | Oracle / Reference |
|---|---|---|---|---|---|
| `sim` | Simulink / Sim Engine | F2 | Implemented | `slx_tests::test_sim_builtin_with_params` | Real SLX Parser / Solver |
| `load_system` | Simulink Engine | F2 | Implemented | `slx_tests::test_sim_builtin_with_params` | Real SLX Package Spec |
| `close_system` | Simulink Engine | F2 | Implemented | `slx_tests::test_sim_builtin_with_params` | Clean-Room Runtime |
| `get_param` | Simulink Engine | F2 | Implemented | `slx_tests::test_sim_builtin_with_params` | XML Block Properties |
| `set_param` | Simulink Engine | F2 | Implemented | `slx_tests::test_sim_builtin_with_params` | Runtime Mutation Store |
| `tf` | Control System | F2 | Implemented | `desktop_tests::test_desktop_session_simulation_and_robotics` | Analytical SISO Model |
| `step` | Control System | F2 | Implemented | `desktop_tests::test_desktop_session_simulation_and_robotics` | Analytical Exponential Integration |
| `fkine_2link` | Robotics & Kinematics | F2 | Implemented | `desktop_tests::test_desktop_session_simulation_and_robotics` | DH Matrix Composition |
| `fkine_3link` | Robotics & Kinematics | F2 | Implemented | `desktop_tests::test_desktop_session_simulation_and_robotics` | Planar Geometric Jacobian |
| `ikine_2link` | Robotics & Kinematics | F2 | Implemented | `desktop_tests::test_desktop_session_simulation_and_robotics` | Geometric Law of Cosines |
| `rotx` / `roty` / `rotz` | Robotics (SO3) | F2 | Implemented | `stdlib_tests::test_elementwise_elementary_math` | Orthogonal Rotation Groups |
| `fitlm` | Machine Learning | F2 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Normal Equations (Faer SVD) |
| `predict` | Machine Learning | F2 | Implemented | `stdlib_tests::test_reductions_and_statistics` | Matrix Vector Inference |
| `fzero` | Optimization | F2 | Partial | `desktop_tests::test_desktop_session_numerical_methods_and_data` | Brent-Dekker Root Finder |
| `fminsearch` | Optimization | F2 | Partial | `desktop_tests::test_desktop_session_numerical_methods_and_data` | Nelder-Mead Simplex |
| `butter` | Signal Processing | F2 | Partial | `stdlib_tests::test_signal_and_fft` | Bilinear Transform IIR |
| `filtfilt` | Signal Processing | F2 | Partial | `stdlib_tests::test_signal_and_fft` | Forward-Backward Zero-Phase |
| `bode` | Control System | F2 | Partial | `desktop_tests::test_desktop_session_simulation_and_robotics` | Frequency Grid Mag/Phase |
| `nyquist` | Control System | F2 | Partial | `desktop_tests::test_desktop_session_simulation_and_robotics` | Complex Polar Plotting |
| `ode45` | Numerical Differential Eq | F2 | Partial | `desktop_tests::test_desktop_session_numerical_methods_and_data` | Dormand-Prince (RK45) Fixed |
| `ode23` | Numerical Differential Eq | F2 | Partial | `desktop_tests::test_desktop_session_numerical_methods_and_data` | Bogacki-Shampine (RK23) |
| `sym` / `solve` | Symbolic Math | F2 | Missing | *Out of Scope by Design* | Clarified in DESIGN.md |
