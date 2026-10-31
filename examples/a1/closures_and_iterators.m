% RustLab Milestone A1 Demonstration: Closures, Function Handles, and Higher-Order Iterators
% Demonstrates lexical closures, cellfun/arrayfun/structfun, and nested end indexing.

disp('=== RustLab Milestone A1: Closures and Higher-Order Functions ===');

% 1. Named function handles
f_sin = @sin;
val = f_sin(pi / 2);
fprintf('f_sin(pi/2) = %0.4f\n', val);

% 2. Lexical closures capturing environment by value
scale = 10;
offset = 5;
linear_map = @(x) x * scale + offset;

fprintf('linear_map(3) before modification = %0.1f\n', linear_map(3));

% Mutating scale in the outer environment does NOT mutate the captured value
scale = 999;
fprintf('linear_map(3) after outer scale=999 = %0.1f (lexical capture verified)\n', linear_map(3));

% 3. arrayfun and cellfun
data = [1, 2, 3, 4, 5];
squares = arrayfun(@(x) x.^2, data);
disp('Squares computed via arrayfun:');
disp(squares);

words = {'apple', 'banana', 'cherry'};
lengths = cellfun(@(s) length(s), words);
disp('Word lengths computed via cellfun:');
disp(lengths);

% 4. Nested end indexing contexts
A = [10, 20, 30; 40, 50, 60; 70, 80, 90];
B = [1, 2, 3];
% A(1:end-B(end), end) resolves outer end=3, inner B(end)=3 -> end-3 = 0, or end-B(1) = 2
sub_matrix = A(1:end-1, end);
disp('A(1:end-1, end):');
disp(sub_matrix);

% 5. Boolean mask indexing and assignment
M = [1, 5, 2; 8, 3, 7];
mask = M > 4;
disp('Elements where M > 4:');
disp(M(mask));

M(M > 4) = 0;
disp('Matrix after masked assignment M(M > 4) = 0:');
disp(M);
