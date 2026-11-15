% 2D Adaptive Quadrature Integral
f = @(x, y) x .* y .* exp(-(x.^2) - (y.^2));
I = integral2(f, 0, 1, 0, 1);
disp('Computed Integral I =');
disp(I);

% Visualization of integrand surface
[X, Y] = meshgrid(0:0.02:1, 0:0.02:1);
Z = f(X, Y);
figure(1);
surf(X, Y, Z);
title('Integrand f(x, y) = x y exp(-x^2 - y^2)');
xlabel('x');
ylabel('y');
zlabel('f(x, y)');
