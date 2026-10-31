% Example: Eigenvalue Decomposition (eig)
% Demonstrates symmetric eig (native Householder + QL) and sign normalization

disp('=== Symmetric 3x3 Matrix Eigenvalues and Eigenvectors ===');
A = [4 1 -2; 1 2 0; -2 0 3];
disp('Matrix A:');
disp(A);

[V, D] = eig(A);
disp('Eigenvectors V (columns):');
disp(V);

disp('Eigenvalue Diagonal D:');
disp(D);

disp('Reconstruction error ||A*V - V*D||:');
err = norm(A * V - V * D);
disp(err);
