% Coupled 3-DOF Mass-Spring System
K = [2 -1 0; -1 2 -1; 0 -1 2];
M = eye(3);
[V, D] = eig(K);
omega = sqrt(diag(D));

figure(1);
stem(1:3, omega);
title('Natural Frequencies \omega_i of Coupled System');
xlabel('Mode Index');
ylabel('Frequency (rad/s)');
grid on;
