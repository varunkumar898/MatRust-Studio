% Chaotic Lorenz Strange Attractor
sigma = 10;
rho = 28;
beta = 8/3;
f = @(t, y) [sigma * (y(2) - y(1)); y(1) * (rho - y(3)) - y(2); y(1) * y(2) - beta * y(3)];
[t, y] = ode45(f, [0, 25], [1; 1; 1]);

figure(1);
plot3(y(:, 1), y(:, 2), y(:, 3));
title('Lorenz Strange Attractor (sigma=10, rho=28, beta=8/3)');
xlabel('X');
ylabel('Y');
zlabel('Z');
grid on;
