% Helmholtz Potential & Vector Gradient
[X, Y] = meshgrid(-2:0.2:2, -2:0.2:2);
Phi = exp(-X.^2 - Y.^2);
[U, V] = gradient(Phi, 0.2, 0.2);

figure(1);
contour(X, Y, Phi);
hold on;
quiver(X, Y, U, V);
hold off;
title('Gaussian Potential and Gradient Field');
xlabel('x');
ylabel('y');
legend('Contours', 'Gradient Vectors');
