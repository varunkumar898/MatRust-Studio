% Example: Singular Value Decomposition (svd)
% Demonstrates economy SVD and reconstruction property

disp('=== Rectangular 4x3 Matrix SVD ===');
B = [1 2 3; 4 5 6; 7 8 9; 10 11 12];
disp('Matrix B (4x3):');
disp(B);

[U, S, V] = svd(B, 'econ');
disp('Left singular vectors U (4x3):');
disp(U);

disp('Singular values S (3x3):');
disp(S);

disp('Right singular vectors V (3x3):');
disp(V);

disp('Reconstruction error ||U*S*V'' - B||:');
err = norm(U * S * V' - B);
disp(err);
