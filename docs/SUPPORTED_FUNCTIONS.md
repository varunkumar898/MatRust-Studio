# Supported MATLAB Functions Catalog: `mlapp-rs`

This document details the built-in function library available in the native runtime, error diagnostic behavior for unsupported functions, and the optional GNU Octave bridge.

---

## 1. Core Built-In Functions

Total registered functions: **225**.

| # | Function Name | Category | Engine |
| :--- | :--- | :--- | :--- |
| 1 | `abs` | Elementary Math & Trig | Native Clean-Room |
| 2 | `acos` | Elementary Math & Trig | Native Clean-Room |
| 3 | `addpath` | General Utility | Native Clean-Room |
| 4 | `all` | Sorting & Searching | Native Clean-Room |
| 5 | `angle` | General Utility | Native Clean-Room |
| 6 | `any` | Sorting & Searching | Native Clean-Room |
| 7 | `arrayfun` | General Utility | Native Clean-Room |
| 8 | `asin` | Elementary Math & Trig | Native Clean-Room |
| 9 | `assert` | General Utility | Native Clean-Room |
| 10 | `atan` | Elementary Math & Trig | Native Clean-Room |
| 11 | `atan2` | Elementary Math & Trig | Native Clean-Room |
| 12 | `bitand` | General Utility | Native Clean-Room |
| 13 | `bitor` | General Utility | Native Clean-Room |
| 14 | `bitshift` | General Utility | Native Clean-Room |
| 15 | `bitxor` | General Utility | Native Clean-Room |
| 16 | `blackman` | Signal Processing | Native Clean-Room |
| 17 | `cast` | General Utility | Native Clean-Room |
| 18 | `cat` | General Utility | Native Clean-Room |
| 19 | `ceil` | Elementary Math & Trig | Native Clean-Room |
| 20 | `cell` | General Utility | Native Clean-Room |
| 21 | `cellfun` | General Utility | Native Clean-Room |
| 22 | `char` | General Utility | Native Clean-Room |
| 23 | `chol` | General Utility | Native Clean-Room |
| 24 | `circshift` | General Utility | Native Clean-Room |
| 25 | `class` | General Utility | Native Clean-Room |
| 26 | `cond` | Linear Algebra | Native Clean-Room |
| 27 | `confusionmat` | Machine Learning | Native Clean-Room |
| 28 | `conj` | General Utility | Native Clean-Room |
| 29 | `contains` | General Utility | Native Clean-Room |
| 30 | `conv` | General Utility | Native Clean-Room |
| 31 | `corrcoef` | Statistics & Reductions | Native Clean-Room |
| 32 | `cos` | Elementary Math & Trig | Native Clean-Room |
| 33 | `cov` | Statistics & Reductions | Native Clean-Room |
| 34 | `cummax` | Statistics & Reductions | Native Clean-Room |
| 35 | `cummin` | Statistics & Reductions | Native Clean-Room |
| 36 | `cumprod` | Statistics & Reductions | Native Clean-Room |
| 37 | `cumsum` | Statistics & Reductions | Native Clean-Room |
| 38 | `cumtrapz` | General Utility | Native Clean-Room |
| 39 | `det` | Linear Algebra | Native Clean-Room |
| 40 | `diag` | Inspection & Shape | Native Clean-Room |
| 41 | `diff` | General Utility | Native Clean-Room |
| 42 | `disp` | Strings & Text | Native Clean-Room |
| 43 | `double` | General Utility | Native Clean-Room |
| 44 | `eig` | Linear Algebra | Native Clean-Room |
| 45 | `endsWith` | General Utility | Native Clean-Room |
| 46 | `eps` | General Utility | Native Clean-Room |
| 47 | `error` | General Utility | Native Clean-Room |
| 48 | `exist` | General Utility | Native Clean-Room |
| 49 | `exp` | Elementary Math & Trig | Native Clean-Room |
| 50 | `expm` | Linear Algebra | Native Clean-Room |
| 51 | `eye` | Generators | Native Clean-Room |
| 52 | `false` | Generators | Native Clean-Room |
| 53 | `fft` | Signal Processing | Native Clean-Room |
| 54 | `fft2` | Signal Processing | Native Clean-Room |
| 55 | `fftshift` | General Utility | Native Clean-Room |
| 56 | `fieldnames` | General Utility | Native Clean-Room |
| 57 | `filter` | Signal Processing | Native Clean-Room |
| 58 | `filtfilt` | Signal Processing | Native Clean-Room |
| 59 | `find` | Sorting & Searching | Native Clean-Room |
| 60 | `fitlm` | Machine Learning | Native Clean-Room |
| 61 | `fix` | Elementary Math & Trig | Native Clean-Room |
| 62 | `fkine_2link` | Robotics | Native Clean-Room |
| 63 | `flintmax` | General Utility | Native Clean-Room |
| 64 | `fliplr` | General Utility | Native Clean-Room |
| 65 | `flipud` | General Utility | Native Clean-Room |
| 66 | `floor` | Elementary Math & Trig | Native Clean-Room |
| 67 | `fprintf` | Strings & Text | Native Clean-Room |
| 68 | `full` | General Utility | Native Clean-Room |
| 69 | `gradient` | General Utility | Native Clean-Room |
| 70 | `hamming` | Signal Processing | Native Clean-Room |
| 71 | `hann` | Signal Processing | Native Clean-Room |
| 72 | `head` | Tables & CSV I/O | Native Clean-Room |
| 73 | `help` | General Utility | Native Clean-Room |
| 74 | `histcounts` | Statistics & Reductions | Native Clean-Room |
| 75 | `horzcat` | General Utility | Native Clean-Room |
| 76 | `idivide` | General Utility | Native Clean-Room |
| 77 | `ifft` | Signal Processing | Native Clean-Room |
| 78 | `ifft2` | Signal Processing | Native Clean-Room |
| 79 | `ikine_2link` | Robotics | Native Clean-Room |
| 80 | `imag` | General Utility | Native Clean-Room |
| 81 | `imfilter` | Image Processing | Native Clean-Room |
| 82 | `imresize` | Image Processing | Native Clean-Room |
| 83 | `int16` | General Utility | Native Clean-Room |
| 84 | `int32` | General Utility | Native Clean-Room |
| 85 | `int64` | General Utility | Native Clean-Room |
| 86 | `int8` | General Utility | Native Clean-Room |
| 87 | `interp1` | General Utility | Native Clean-Room |
| 88 | `intmax` | General Utility | Native Clean-Room |
| 89 | `intmin` | General Utility | Native Clean-Room |
| 90 | `inv` | Linear Algebra | Native Clean-Room |
| 91 | `ipermute` | General Utility | Native Clean-Room |
| 92 | `isa` | General Utility | Native Clean-Room |
| 93 | `isempty` | Inspection & Shape | Native Clean-Room |
| 94 | `isfield` | General Utility | Native Clean-Room |
| 95 | `isfinite` | General Utility | Native Clean-Room |
| 96 | `isfloat` | General Utility | Native Clean-Room |
| 97 | `isinf` | General Utility | Native Clean-Room |
| 98 | `isinteger` | General Utility | Native Clean-Room |
| 99 | `ismatrix` | Inspection & Shape | Native Clean-Room |
| 100 | `isnan` | General Utility | Native Clean-Room |
| 101 | `isnumeric` | General Utility | Native Clean-Room |
| 102 | `isscalar` | Inspection & Shape | Native Clean-Room |
| 103 | `issparse` | General Utility | Native Clean-Room |
| 104 | `isvector` | Inspection & Shape | Native Clean-Room |
| 105 | `jtraj` | Robotics | Native Clean-Room |
| 106 | `kmeans` | Machine Learning | Native Clean-Room |
| 107 | `kron` | Linear Algebra | Native Clean-Room |
| 108 | `length` | Inspection & Shape | Native Clean-Room |
| 109 | `linspace` | Generators | Native Clean-Room |
| 110 | `load` | General Utility | Native Clean-Room |
| 111 | `log` | Elementary Math & Trig | Native Clean-Room |
| 112 | `log10` | Elementary Math & Trig | Native Clean-Room |
| 113 | `log2` | Elementary Math & Trig | Native Clean-Room |
| 114 | `logical` | General Utility | Native Clean-Room |
| 115 | `logspace` | Generators | Native Clean-Room |
| 116 | `lower` | Strings & Text | Native Clean-Room |
| 117 | `lu` | Linear Algebra | Native Clean-Room |
| 118 | `magic` | Generators | Native Clean-Room |
| 119 | `mat2str` | Strings & Text | Native Clean-Room |
| 120 | `max` | Statistics & Reductions | Native Clean-Room |
| 121 | `mean` | Statistics & Reductions | Native Clean-Room |
| 122 | `median` | Statistics & Reductions | Native Clean-Room |
| 123 | `meshgrid` | General Utility | Native Clean-Room |
| 124 | `min` | Statistics & Reductions | Native Clean-Room |
| 125 | `mldivide` | Linear Algebra | Native Clean-Room |
| 126 | `mod` | Elementary Math & Trig | Native Clean-Room |
| 127 | `mode` | Statistics & Reductions | Native Clean-Room |
| 128 | `movmean` | Statistics & Reductions | Native Clean-Room |
| 129 | `movmedian` | Statistics & Reductions | Native Clean-Room |
| 130 | `mrdivide` | Linear Algebra | Native Clean-Room |
| 131 | `ndgrid` | General Utility | Native Clean-Room |
| 132 | `ndims` | Inspection & Shape | Native Clean-Room |
| 133 | `nnz` | General Utility | Native Clean-Room |
| 134 | `nonzeros` | General Utility | Native Clean-Room |
| 135 | `norm` | General Utility | Native Clean-Room |
| 136 | `null` | Linear Algebra | Native Clean-Room |
| 137 | `num2str` | Strings & Text | Native Clean-Room |
| 138 | `numel` | Inspection & Shape | Native Clean-Room |
| 139 | `ones` | Generators | Native Clean-Room |
| 140 | `orth` | Linear Algebra | Native Clean-Room |
| 141 | `permute` | General Utility | Native Clean-Room |
| 142 | `pinv` | Linear Algebra | Native Clean-Room |
| 143 | `polyfit` | General Utility | Native Clean-Room |
| 144 | `polyval` | General Utility | Native Clean-Room |
| 145 | `prctile` | Statistics & Reductions | Native Clean-Room |
| 146 | `predict` | Machine Learning | Native Clean-Room |
| 147 | `prod` | Statistics & Reductions | Native Clean-Room |
| 148 | `qr` | Linear Algebra | Native Clean-Room |
| 149 | `quantile` | Statistics & Reductions | Native Clean-Room |
| 150 | `rand` | Generators | Native Clean-Room |
| 151 | `randn` | Generators | Native Clean-Room |
| 152 | `rank` | Linear Algebra | Native Clean-Room |
| 153 | `readmatrix` | Tables & CSV I/O | Native Clean-Room |
| 154 | `readtable` | Tables & CSV I/O | Native Clean-Room |
| 155 | `real` | General Utility | Native Clean-Room |
| 156 | `realmax` | General Utility | Native Clean-Room |
| 157 | `realmin` | General Utility | Native Clean-Room |
| 158 | `regexp` | Strings & Text | Native Clean-Room |
| 159 | `regexprep` | Strings & Text | Native Clean-Room |
| 160 | `rem` | Elementary Math & Trig | Native Clean-Room |
| 161 | `repmat` | General Utility | Native Clean-Room |
| 162 | `reshape` | General Utility | Native Clean-Room |
| 163 | `rgb2gray` | Image Processing | Native Clean-Room |
| 164 | `rmpath` | General Utility | Native Clean-Room |
| 165 | `roots` | General Utility | Native Clean-Room |
| 166 | `rot90` | General Utility | Native Clean-Room |
| 167 | `rotx` | Robotics | Native Clean-Room |
| 168 | `roty` | Robotics | Native Clean-Room |
| 169 | `rotz` | Robotics | Native Clean-Room |
| 170 | `round` | Elementary Math & Trig | Native Clean-Room |
| 171 | `save` | General Utility | Native Clean-Room |
| 172 | `sign` | Elementary Math & Trig | Native Clean-Room |
| 173 | `sin` | Elementary Math & Trig | Native Clean-Room |
| 174 | `single` | General Utility | Native Clean-Room |
| 175 | `size` | Inspection & Shape | Native Clean-Room |
| 176 | `sort` | Sorting & Searching | Native Clean-Room |
| 177 | `sortrows` | Sorting & Searching | Native Clean-Room |
| 178 | `sparse` | General Utility | Native Clean-Room |
| 179 | `speye` | General Utility | Native Clean-Room |
| 180 | `spones` | General Utility | Native Clean-Room |
| 181 | `sprintf` | Strings & Text | Native Clean-Room |
| 182 | `sqrt` | Elementary Math & Trig | Native Clean-Room |
| 183 | `squeeze` | Inspection & Shape | Native Clean-Room |
| 184 | `startsWith` | General Utility | Native Clean-Room |
| 185 | `std` | Statistics & Reductions | Native Clean-Room |
| 186 | `step` | General Utility | Native Clean-Room |
| 187 | `strcmp` | Strings & Text | Native Clean-Room |
| 188 | `strcmpi` | Strings & Text | Native Clean-Room |
| 189 | `string` | General Utility | Native Clean-Room |
| 190 | `strjoin` | General Utility | Native Clean-Room |
| 191 | `strlength` | General Utility | Native Clean-Room |
| 192 | `strncmp` | Strings & Text | Native Clean-Room |
| 193 | `strrep` | Strings & Text | Native Clean-Room |
| 194 | `strsplit` | General Utility | Native Clean-Room |
| 195 | `strtrim` | Strings & Text | Native Clean-Room |
| 196 | `struct` | General Utility | Native Clean-Room |
| 197 | `structfun` | General Utility | Native Clean-Room |
| 198 | `sum` | Statistics & Reductions | Native Clean-Room |
| 199 | `summary` | Tables & CSV I/O | Native Clean-Room |
| 200 | `svd` | Linear Algebra | Native Clean-Room |
| 201 | `tail` | Tables & CSV I/O | Native Clean-Room |
| 202 | `tan` | Elementary Math & Trig | Native Clean-Room |
| 203 | `tf` | General Utility | Native Clean-Room |
| 204 | `trace` | General Utility | Native Clean-Room |
| 205 | `transl` | Robotics | Native Clean-Room |
| 206 | `trapz` | General Utility | Native Clean-Room |
| 207 | `trotx` | Robotics | Native Clean-Room |
| 208 | `troty` | Robotics | Native Clean-Room |
| 209 | `trotz` | Robotics | Native Clean-Room |
| 210 | `true` | Generators | Native Clean-Room |
| 211 | `uint16` | General Utility | Native Clean-Room |
| 212 | `uint32` | General Utility | Native Clean-Room |
| 213 | `uint64` | General Utility | Native Clean-Room |
| 214 | `uint8` | General Utility | Native Clean-Room |
| 215 | `unique` | Sorting & Searching | Native Clean-Room |
| 216 | `upper` | Strings & Text | Native Clean-Room |
| 217 | `var` | Statistics & Reductions | Native Clean-Room |
| 218 | `vertcat` | General Utility | Native Clean-Room |
| 219 | `warning` | General Utility | Native Clean-Room |
| 220 | `which` | General Utility | Native Clean-Room |
| 221 | `writematrix` | Tables & CSV I/O | Native Clean-Room |
| 222 | `writetable` | Tables & CSV I/O | Native Clean-Room |
| 223 | `xcorr` | Signal Processing | Native Clean-Room |
| 224 | `zeros` | Generators | Native Clean-Room |
| 225 | `zscore` | Statistics & Reductions | Native Clean-Room |

---

## 2. App Designer UI & Graphics Built-Ins (`mlapp-runtime`)

The App Designer runtime environment provides dedicated built-in functions for interactive UI figure control and Cartesian 2D plotting:

| Built-In | Description | Dispatched UI Mutation |
| :--- | :--- | :--- |
| `plot` | 2D Cartesian line plotting on `UIAxes` | `UiMutation::PlotCall` |
| `hold` | Toggles plot hold mode ('on'/'off') | `UiMutation::PlotCall` |
| `uialert` | Modal diagnostic dialog prompt | `UiMutation::Alert` |
| `drawnow` | Flushes pending UI mutation queue | Synchronous event flush |
| `pause` | Cooperatively yields execution with cancellation checks | Yield / Sleep |

---

## 3. Unsupported Function Policy

Unsupported functions **never fail silently**. Calling an unrecognized function immediately raises a loud diagnostic:

```
RuntimeError: Unsupported MATLAB function '<name>' (not implemented in native runtime).
```

---

## 4. GNU Octave Bridge (Tier 4) — NOT IMPLEMENTED

The GNU Octave bridge is currently **NOT IMPLEMENTED** (documented stub). In current versions, unsupported functions fail loudly with a runtime error diagnostic.
