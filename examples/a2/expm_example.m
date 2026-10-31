% Example: Matrix Exponential (expm)
% Demonstrates scaling and squaring with Padé(6,6) approximation

disp('=== Matrix Exponential: 2D Rotation Generator ===');
t = 0.7853981633974483; % pi / 4
A = [0 -t; t 0];
disp('Generator matrix A:');
disp(A);

E = expm(A);
disp('expm(A) (Rotation matrix for pi/4):');
disp(E);

disp('Theoretical rotation matrix:');
R_expected = [cos(t) -sin(t); sin(t) cos(t)];
disp(R_expected);

disp('Error ||expm(A) - R_expected||:');
err = norm(E - R_expected);
disp(err);
