% RustLab Milestone A3: N-Dimensional Array Parity Example
disp('=== RustLab A3: N-Dimensional Arrays Demonstration ===');

% 1. 3D Array Reshape with Unknown Dimension []
A = reshape(1:24, [2, 3, 4]);
disp('Created 2x3x4 tensor with size:');
disp(size(A));

% 2. Multi-subscript Slicing
B = A(1:2, 2:3, 1:2);
disp('Slice A(1:2, 2:3, 1:2) shape:');
disp(size(B));

% 3. Reductions along Dimension 3
S3 = sum(A, 3);
disp('Sum along 3rd dimension (shape 2x3):');
disp(S3);

% 4. 3D Permutation
P = permute(A, [2, 1, 3]);
disp('Permuted dimensions [2, 1, 3] shape:');
disp(size(P));

% 5. Coordinate Grid Generation
[X, Y] = meshgrid(1:3, 1:2);
disp('2D Meshgrid X coordinates:');
disp(X);
disp('2D Meshgrid Y coordinates:');
disp(Y);
