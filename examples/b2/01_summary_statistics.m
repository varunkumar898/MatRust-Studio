% Example 1: Summary Statistics and Reductions in RustLab B2.1
disp('=== Summary Statistics and Reductions ===');

x = [12.5, 18.2, 14.1, 22.0, 19.8, 14.1, 16.5];
disp('Data vector x:');
disp(x);

disp('Median of x:');
disp(median(x));

disp('Mode of x:');
disp(mode(x));

disp('Variance of x (N-1 normalized):');
disp(var(x, 0));

disp('Standard deviation of x:');
disp(std(x));

disp('25th, 50th, 75th percentiles:');
disp(prctile(x, [25, 50, 75]));

disp('Z-score standardized x:');
disp(zscore(x));

% With NaNs and omitnan
y = [10, NaN, 20, 30, NaN];
disp('Data with NaNs y:');
disp(y);

disp('Median with omitnan:');
disp(median(y, 'omitnan'));

disp('Variance with omitnan:');
disp(var(y, 0, 'omitnan'));
