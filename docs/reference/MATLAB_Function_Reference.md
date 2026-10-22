# MATLAB Function Reference (Core + Common Toolbox Functions)

> Source document established 2026-10-22 from the project brief. This file is the
> authoritative compatibility target for `matlab-compatible-rust`. Every entry below
> is a requirement that must be accounted for in `docs/function-coverage.md`.
> Format: `syntax` - what it does. Use `doc name` in MATLAB for the full official page.

## 1. Creating arrays and matrices
| Function | Description |
|---|---|
| `zeros(m,n)` | m-by-n matrix of zeros |
| `ones(m,n)` | m-by-n matrix of ones |
| `eye(n)` | n-by-n identity matrix |
| `rand(m,n)` | uniform random numbers in (0,1) |
| `randn(m,n)` | normally distributed random numbers |
| `randi(max,m,n)` | random integers from 1 to max |
| `randperm(n)` | random permutation of 1..n |
| `linspace(a,b,n)` | n evenly spaced points from a to b |
| `logspace(a,b,n)` | n log-spaced points from 10^a to 10^b |
| `colon operator` | `a:step:b` - range with step |
| `magic(n)` | magic square |
| `diag(v)` | build diagonal matrix, or extract diagonal |
| `repmat(A,m,n)` | tile copies of A |
| `meshgrid(x,y)` | 2-D grid coordinates |
| `ndgrid(x,y)` | N-D grid coordinates |
| `cat(dim,A,B)` | concatenate along dimension |
| `horzcat`, `vertcat` | concatenate horizontally / vertically |
| `blkdiag(A,B)` | block diagonal matrix |

## 2. Array information and manipulation
| Function | Description |
|---|---|
| `size(A)` | dimensions of A |
| `length(A)` | longest dimension |
| `numel(A)` | total element count |
| `ndims(A)` | number of dimensions |
| `isempty(A)` | true if A is empty |
| `reshape(A,m,n)` | change shape, same data |
| `squeeze(A)` | remove singleton dimensions |
| `permute(A,order)` | rearrange dimensions |
| `transpose(A)` / `A'` | transpose |
| `flip`, `fliplr`, `flipud` | reverse order of elements |
| `rot90(A)` | rotate 90 degrees |
| `circshift(A,k)` | circularly shift elements |
| `sort(A)` | sort ascending/descending |
| `sortrows(A,col)` | sort rows by column |
| `unique(A)` | unique values |
| `find(cond)` | indices of nonzero/true elements |
| `any(A)`, `all(A)` | true if any / all elements nonzero |
| `ismember(a,B)` | membership test |
| `setdiff`, `union`, `intersect` | set operations |
| `cumsum`, `cumprod` | cumulative sum / product |
| `diff(A)` | differences between adjacent elements |
| `histcounts(x,edges)` | bin counts |
| `end` | last index inside indexing |
| `triu`, `tril` | upper / lower triangular part |

## 3. Elementary math
| Function | Description |
|---|---|
| `abs`, `sign` | absolute value, sign |
| `sqrt`, `nthroot(x,n)` | square root, nth root |
| `exp`, `log`, `log2`, `log10` | exponential and logarithms |
| `power(a,b)` / `.^` | element-wise power |
| `sin`, `cos`, `tan` | trig (radians) |
| `sind`, `cosd`, `tand` | trig (degrees) |
| `asin`, `acos`, `atan`, `atan2(y,x)` | inverse trig |
| `sinh`, `cosh`, `tanh` | hyperbolic |
| `deg2rad`, `rad2deg` | angle conversion |
| `hypot(a,b)` | sqrt(a^2+b^2) without overflow |
| `floor`, `ceil`, `round`, `fix` | rounding variants |
| `mod(a,b)`, `rem(a,b)` | modulus and remainder |
| `gcd`, `lcm` | greatest common divisor, least common multiple |
| `factorial(n)`, `nchoosek(n,k)` | factorial, combinations |
| `primes(n)`, `isprime(n)`, `factor(n)` | prime numbers |
| `real`, `imag`, `conj`, `angle` | complex number parts |
| `bitand`, `bitor`, `bitxor`, `bitshift` | bitwise operations |
| `dec2bin`, `bin2dec`, `dec2hex`, `hex2dec` | base conversion |

## 4. Statistics
| Function | Description |
|---|---|
| `sum`, `prod` | sum / product |
| `mean`, `median`, `mode` | central tendency |
| `std`, `var` | spread |
| `min`, `max` | extremes (can return index) |
| `range`, `iqr` | spread measures |
| `prctile`, `quantile` | percentiles / quantiles |
| `corrcoef`, `corr`, `cov` | correlation and covariance |
| `movmean`, `movmedian`, `movsum` | moving window statistics |
| `normalize` | normalize data |
| `rescale` | scale to a range |
| `cummax`, `cummin` | running max / min |

## 5. Linear algebra
| Function | Description |
|---|---|
| `A*B`, `A.*B` | matrix multiply, element-wise multiply |
| `A\b` | solve linear system Ax=b |
| `inv(A)`, `pinv(A)` | inverse, pseudo-inverse |
| `det(A)` | determinant |
| `rank(A)` | rank |
| `trace(A)` | sum of diagonal |
| `norm(A)` | matrix / vector norm |
| `eig(A)` | eigenvalues / eigenvectors |
| `svd(A)` | singular value decomposition |
| `lu`, `qr`, `chol` | factorizations |
| `null(A)`, `orth(A)` | null space, orthonormal basis |
| `expm(A)` | matrix exponential |
| `cross(a,b)`, `dot(a,b)`, `kron(A,B)` | vector / Kronecker products |
| `cond(A)` | condition number |

## 6. Logical and relational
| Function | Description |
|---|---|
| `==`, `~=`, `<`, `>`, `<=`, `>=` | comparisons |
| `&`, `\|`, `~`, `xor` | element-wise logic |
| `&&`, `\|\|` | short-circuit logic |
| `isnan`, `isinf`, `isfinite` | check special values |
| `isequal(a,b)` | equality of whole arrays |
| `islogical`, `isnumeric`, `ischar`, `iscell`, `isstruct` | type checks |
| `isa(x,'class')` | class check |

## 7. Strings and text
| Function | Description |
|---|---|
| `string`, `char` | convert to string / char array |
| `strcat`, `append` | join text |
| `strsplit`, `split` | split text |
| `strjoin` | join cell array of strings |
| `strrep`, `replace` | replace substrings |
| `strfind`, `contains`, `startsWith`, `endsWith` | search |
| `strcmp`, `strcmpi`, `strncmp` | compare |
| `upper`, `lower`, `strtrim` | case / whitespace |
| `num2str`, `str2num`, `str2double` | number and text conversion |
| `sprintf`, `fprintf`, `disp` | formatted output |
| `regexp`, `regexprep` | regular expressions |
| `extractBetween`, `extractAfter`, `extractBefore` | pull out text |
| `strlength` | string length |
| `input`, `inputdlg` | read user input |

## 8. Data types and containers
| Function | Description |
|---|---|
| `int8`..`int64`, `uint8`..`uint64` | integer types (important for embedded) |
| `single`, `double` | floating point |
| `class(x)`, `whos` | inspect type / workspace |
| `cast`, `typecast` | change type (typecast reinterprets bits) |
| `intmax`, `intmin`, `realmax`, `eps` | numeric limits |
| `cell`, `cell2mat`, `num2cell` | cell arrays |
| `cellfun`, `arrayfun` | apply function across elements |
| `struct`, `fieldnames`, `isfield`, `rmfield` | structures |
| `table`, `array2table`, `table2array` | tables |
| `height(T)`, `width(T)` | table size |
| `innerjoin`, `outerjoin`, `join` | table joins |
| `groupsummary`, `varfun` | group and summarize |
| `containers.Map` | dictionary / key-value store |
| `dictionary` | newer key-value type |
| `datetime`, `duration`, `timetable` | time data |
| `categorical` | categories |
| `fi`, `fixdt` | fixed-point types (Fixed-Point Designer) |

## 9. Programming and control flow
| Function / keyword | Description |
|---|---|
| `if`, `elseif`, `else`, `end` | conditionals |
| `for`, `while`, `break`, `continue` | loops |
| `switch`, `case`, `otherwise` | multi-branch |
| `try`, `catch` | error handling |
| `function` | define a function |
| `return` | exit function |
| `nargin`, `nargout` | argument counts |
| `varargin`, `varargout` | variable arguments |
| `inputParser`, `validateattributes`, `arguments` | input validation |
| `error`, `warning`, `assert` | raise errors / warnings / checks |
| `anonymous function` | `@(x) x.^2` |
| `feval`, `func2str`, `str2func` | function handles |
| `tic`, `toc` | timing |
| `pause(sec)` | wait |
| `parfor`, `parfeval` | parallel loops (Parallel Computing Toolbox) |
| `classdef`, `properties`, `methods` | object-oriented classes |

## 10. Files and data I/O
| Function | Description |
|---|---|
| `load`, `save` | MAT-file workspace I/O |
| `readtable`, `writetable` | CSV / Excel to table |
| `readmatrix`, `writematrix` | numeric files |
| `readcell` | read mixed data into cell array |
| `csvread`, `dlmread` | older delimited readers |
| `xlsread`, `xlswrite` | Excel (older; prefer readtable) |
| `fopen`, `fclose` | open / close a file |
| `fread`, `fwrite` | binary read / write |
| `fgetl`, `fgets`, `fscanf`, `fprintf` | text file I/O |
| `textscan` | parse formatted text |
| `imread`, `imwrite` | images |
| `audioread`, `audiowrite` | audio |
| `jsondecode`, `jsonencode` | JSON |
| `webread`, `webwrite` | web / REST APIs |
| `dir`, `ls`, `pwd`, `cd`, `mkdir`, `exist`, `delete` | file system |
| `fullfile`, `fileparts` | path handling |

## 11. Plotting and visualization
| Function | Description |
|---|---|
| `plot(x,y)` | 2-D line plot |
| `scatter(x,y)` | scatter plot |
| `bar`, `barh` | bar chart |
| `histogram(x)` | histogram |
| `pie` | pie chart |
| `stairs`, `stem` | stairstep / discrete sequence plots |
| `semilogx`, `semilogy`, `loglog` | log-scale plots |
| `errorbar` | error bars |
| `area`, `fill` | filled plots |
| `polarplot` | polar plot |
| `plot3`, `scatter3` | 3-D line / scatter |
| `surf`, `mesh`, `contour`, `contourf` | surfaces and contours |
| `quiver` | vector field |
| `imagesc`, `imshow`, `heatmap` | matrix / image display |
| `subplot`, `tiledlayout`, `nexttile` | multiple axes |
| `figure`, `close`, `clf`, `hold on/off` | figure management |
| `xlabel`, `ylabel`, `zlabel`, `title`, `legend` | labels |
| `xlim`, `ylim`, `axis`, `grid`, `colorbar`, `colormap` | axis control |
| `text`, `annotation` | add text / shapes |
| `saveas`, `exportgraphics`, `print` | export figures |
| `drawnow` | force graphics update |
| `animatedline`, `comet` | animation |

## 12. Polynomials, interpolation, calculus, optimization
| Function | Description |
|---|---|
| `polyfit(x,y,n)`, `polyval(p,x)` | fit / evaluate polynomial |
| `roots(p)`, `poly(r)` | polynomial roots / from roots |
| `conv`, `deconv` | polynomial multiply / divide (also convolution) |
| `polyder`, `polyint` | polynomial derivative / integral |
| `interp1`, `interp2`, `spline`, `pchip` | interpolation |
| `trapz`, `cumtrapz` | numerical integration from data |
| `integral`, `integral2`, `quad` | integrate a function |
| `gradient`, `del2` | numerical derivatives |
| `fzero` | root of a scalar function |
| `fsolve` | solve nonlinear systems (Optimization Toolbox) |
| `fminsearch`, `fminunc`, `fmincon` | minimization |
| `lsqcurvefit`, `lsqnonlin` | curve fitting / least squares |
| `linprog`, `quadprog` | linear / quadratic programming |
| `ode45`, `ode23`, `ode15s` | ODE solvers |
| `fit`, `cftool` | curve fitting (Curve Fitting Toolbox) |

## 13. Signal processing (Signal Processing Toolbox)
| Function | Description |
|---|---|
| `fft`, `ifft`, `fftshift` | Fourier transform |
| `fft2`, `ifft2` | 2-D FFT |
| `filter(b,a,x)` | apply digital filter |
| `filtfilt` | zero-phase filtering |
| `conv`, `xcorr` | convolution, correlation |
| `butter`, `cheby1`, `cheby2`, `ellip` | IIR filter design |
| `fir1`, `firpm` | FIR filter design |
| `freqz`, `freqs` | frequency response |
| `designfilt` | interactive filter design |
| `spectrogram`, `pwelch`, `periodogram` | spectral analysis |
| `resample`, `decimate`, `interp` | sample-rate change |
| `detrend`, `smoothdata` | cleaning data |
| `findpeaks` | peak detection |
| `hilbert`, `envelope` | analytic signal / envelope |
| `window`, `hamming`, `hann` | window functions |
| `chirp`, `square`, `sawtooth` | signal generators |
| `snr`, `rms`, `bandpower` | measurements |

## 14. Control systems (Control System Toolbox)
| Function | Description |
|---|---|
| `tf(num,den)` | transfer function |
| `ss(A,B,C,D)` | state-space model |
| `zpk` | zero-pole-gain model |
| `c2d`, `d2c` | continuous / discrete conversion |
| `step`, `impulse`, `lsim` | time responses |
| `bode`, `nyquist`, `nichols` | frequency responses |
| `margin` | gain and phase margins |
| `rlocus` | root locus |
| `pole`, `zero`, `pzmap` | poles and zeros |
| `feedback`, `series`, `parallel` | connect systems |
| `pid`, `pidtune` | PID controllers |
| `lqr`, `place`, `kalman` | state-feedback and estimator design |
| `ctrb`, `obsv` | controllability / observability |
| `stepinfo` | rise time, overshoot, settling time |

## 15. Hardware, serial and networking
| Function | Description |
|---|---|
| `serialportlist` | list available serial ports |
| `serialport(port,baud)` | open serial connection |
| `write`, `writeline` | send data |
| `read`, `readline`, `readbinblock` | receive data |
| `flush`, `configureTerminator`, `configureCallback` | serial setup |
| `arduino(port,board)` | connect to Arduino (support package) |
| `readDigitalPin`, `writeDigitalPin`, `readVoltage`, `writePWMDutyCycle` | Arduino pin I/O |
| `i2cdev`, `device`, `spidev` | I2C / SPI devices |
| `raspi` | Raspberry Pi connection |
| `tcpclient`, `tcpserver` | TCP/IP |
| `udpport` | UDP |
| `bluetooth`, `ble` | Bluetooth |
| `mqttclient` | MQTT (IoT) |
| `thingSpeakRead`, `thingSpeakWrite` | ThingSpeak IoT |
| `can`, `canChannel` | CAN bus (Vehicle Network Toolbox) |
| `visadev` | instrument control (Instrument Control Toolbox) |

## 16. Code generation and Simulink
| Function | Description |
|---|---|
| `codegen` | MATLAB Coder: generate C/C++ |
| `coder.config` | code generation settings |
| `coder.typeof`, `coder.varsize` | specify types / variable sizes |
| `slbuild` | build a Simulink model for a target |
| `sim`, `simset` | run Simulink model from script |
| `open_system`, `new_system`, `add_block`, `add_line` | build models programmatically |
| `set_param`, `get_param` | model / block parameters |
| `bdclose` | close model |

## 17. Symbolic Math (Symbolic Math Toolbox)
| Function | Description |
|---|---|
| `syms x y` | declare symbolic variables |
| `solve`, `vpasolve` | solve equations |
| `diff`, `int`, `limit`, `taylor` | calculus |
| `simplify`, `expand`, `factor`, `collect` | algebra |
| `subs`, `double` | substitute / convert to number |
| `laplace`, `ilaplace`, `fourier`, `ifourier`, `ztrans` | transforms |
| `dsolve` | solve ODEs symbolically |
| `matlabFunction` | convert symbolic to function |

## 18. Machine learning and deep learning
| Function | Description |
|---|---|
| `fitlm`, `fitglm` | linear / generalized linear models |
| `fitctree`, `fitrtree` | decision trees |
| `fitcsvm` | support vector machine |
| `fitcknn` | k-nearest neighbors |
| `fitcensemble`, `TreeBagger` | ensembles / random forest |
| `kmeans`, `kmedoids`, `dbscan` | clustering |
| `pca` | principal component analysis |
| `predict` | apply trained model |
| `crossval`, `kfoldLoss` | cross-validation |
| `confusionmat`, `confusionchart` | classification evaluation |
| `trainNetwork`, `trainnet`, `dlnetwork` | deep learning training |
| `layerGraph`, `imageInputLayer`, `convolution2dLayer` | build network layers |
| `trainingOptions` | training settings |
| `classify`, `activations` | deep learning inference |
| `importONNXNetwork` | bring in ONNX models |

## 19. Dates and time
| Function | Description |
|---|---|
| `datetime('now')` | current date/time |
| `datestr`, `datenum` | older date formats |
| `seconds`, `minutes`, `hours`, `days` | durations |
| `between`, `caldiff` | differences between dates |
| `weekday`, `month`, `year` | extract parts |
| `retime`, `synchronize` | resample timetables |

## 20. Help, debugging and workspace
| Function | Description |
|---|---|
| `help name`, `doc name` | built-in documentation |
| `lookfor keyword` | search help by keyword |
| `which name` | locate a function |
| `what`, `who`, `whos` | list files / variables |
| `clear`, `clc`, `close all` | reset workspace / console / figures |
| `format long/short` | display precision |
| `dbstop`, `dbstep`, `dbcont`, `dbquit` | debugger commands |
| `profile on/viewer` | profiler |
| `checkcode` | static code analysis |
| `ver`, `version` | installed toolboxes / version |
| `addpath`, `rmpath`, `path` | search path |
| `publish`, `export` | export live scripts / reports |
| `matlab.unittest` | unit testing framework |

---

### Compatibility notes

- MATLAB indexing starts at 1, not 0.
- Operators prefixed with `.` (`.*`, `./`, `.^`) are element-wise; without the dot
  they are matrix operations.
- Functions marked with a toolbox name need that toolbox licensed and installed;
  check with `ver`.
- Function names and options change between releases; confirm exact syntax on
  mathworks.com/help for the user's release.
