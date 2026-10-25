% RustLab Milestone A3: Compressed Sparse Column (CSC) Example
disp('=== RustLab A3: Sparse Matrices Demonstration ===');

% 1. Sparse Matrix with Duplicate Coordinate Accumulation
i = [1 2 2 3];
j = [1 2 2 3];
v = [10 20 30 40];
S = sparse(i, j, v, 3, 3);
disp('Constructed sparse matrix S:');
disp(S);

disp('Dense representation full(S):');
disp(full(S));

% 2. Sparse Identity and Nonzeros
I_sp = speye(3);
disp('Sparse identity speye(3):');
disp(I_sp);

disp('Number of nonzeros nnz(S):');
disp(nnz(S));

% 3. Sparse Algebra
S2 = S + I_sp;
disp('Sparse addition S + speye(3):');
disp(full(S2));
