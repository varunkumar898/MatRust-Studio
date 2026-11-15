% Runge Interpolation: Spline vs PCHIP
x_pts = linspace(-1, 1, 9);
y_pts = 1 ./ (1 + 25 * x_pts.^2);
x_fine = linspace(-1, 1, 200);

y_spline = spline(x_pts, y_pts, x_fine);
y_pchip = pchip(x_pts, y_pts, x_fine);
y_exact = 1 ./ (1 + 25 * x_fine.^2);

figure(1);
plot(x_fine, y_exact);
hold on;
plot(x_fine, y_spline);
plot(x_fine, y_pchip);
plot(x_pts, y_pts);
hold off;
title('Interpolation of Runge Function: Spline vs PCHIP');
xlabel('x');
ylabel('y');
legend('Exact Runge', 'Cubic Spline', 'PCHIP Hermite', 'Sample Points');
grid on;
