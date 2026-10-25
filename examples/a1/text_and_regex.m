% RustLab Milestone A1 Demonstration: Text Processing, Regex, and Format Recycling
% Demonstrates sprintf format recycling, regexp with named capture groups, and diagnostics.

disp('=== RustLab Milestone A1: Text, Regex, and Diagnostics ===');

% 1. sprintf with format recycling across column-major 2D matrix
matrix = [1.2, 3.4; 5.6, 7.8];
table_str = sprintf('Val A: %0.2f | Val B: %0.2f\n', matrix);
disp('Recycled format output:');
fprintf('%s', table_str);

% 2. String manipulation: strsplit, strjoin, strrep
csv_line = 'engine_rpm,oil_pressure,coolant_temp';
fields = strsplit(csv_line, ',');
header = strjoin(fields, ' -> ');
disp('Joined header:');
disp(header);

% 3. Regular expressions with named capture groups
log_line = 'PID: 4096, STATUS: ACTIVE';
st = regexp(log_line, 'PID: (?<pid>\d+), STATUS: (?<status>\w+)', 'names');
fprintf('Parsed PID: %s, STATUS: %s\n', st.pid, st.status);

% 4. Regular expression replacement
cleaned = regexprep('temperature=350K; pressure=101kPa', '(\w+)=(\w+)', '$1 is $2');
disp('Regexprep result:');
disp(cleaned);

% 5. IEEE-754 NaN/Inf detection
measurements = [23.5, NaN, 24.1, Inf, -Inf];
nan_count = sum(isnan(measurements));
inf_count = sum(isinf(measurements));
fprintf('Detected NaNs: %d, Infs: %d\n', nan_count, inf_count);

% 6. Diagnostics and Assertion
assert(nan_count == 1, 'Validation error: expected exactly 1 NaN');
disp('All Milestone A1 assertions passed successfully.');
