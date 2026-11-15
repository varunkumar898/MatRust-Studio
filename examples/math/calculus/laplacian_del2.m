% Discrete Laplacian of Thermal Field
[X, Y] = meshgrid(0:0.05:1, 0:0.05:1);
T = sin(pi * X) .* sin(pi * Y);
L = del2(T, 0.05, 0.05);

figure(1);
mesh(X, Y, L);
title('Discrete Laplacian del2 of Temperature Field');
xlabel('x');
ylabel('y');
zlabel('del2(T)');
