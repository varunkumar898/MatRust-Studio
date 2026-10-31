% Example: Moore-Penrose Pseudoinverse (pinv)
% Demonstrates pseudoinverse on a rank-deficient matrix

disp('=== Rank-Deficient 3x3 Pseudoinverse ===');
A = [1 2 3; 2 4 6; 1 1 1];
disp('Matrix A (rank 2):');
disp(A);

disp('Rank:');
disp(rank(A));

P = pinv(A);
disp('Pseudoinverse pinv(A):');
disp(P);

disp('Moore-Penrose Condition 1: ||A*P*A - A||:');
err1 = norm(A * P * A - A);
disp(err1);

disp('Moore-Penrose Condition 2: ||P*A*P - P||:');
err2 = norm(P * A * P - P);
disp(err2);
