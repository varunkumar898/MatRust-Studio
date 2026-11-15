% Overdetermined Least Squares
x_pts = [0.1; 0.5; 1.0; 1.5; 2.0; 2.5; 3.0];
y_pts = [1.2; 1.8; 2.9; 4.7; 7.1; 9.8; 13.5];

c = polyfit(x_pts, y_pts, 2);
x_fine = linspace(0, 3, 100);
y_fit = polyval(c, x_fine);

figure(1);
plot(x_fine, y_fit);
hold on;
plot(x_pts, y_pts);
hold off;
title('Quadratic Least Squares Fit via QR');
xlabel('x');
ylabel('y');
legend('Fitted Model', 'Data Points');
grid on;
