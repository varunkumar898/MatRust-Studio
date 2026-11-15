% 2D Vortex Flow Field
[X, Y] = meshgrid(-2:0.25:2, -2:0.25:2);
R2 = X.^2 + Y.^2 + 0.1;
U = -Y ./ R2;
V = X ./ R2;

figure(1);
quiver(X, Y, U, V);
title('2D Circulating Vortex Velocity Field');
xlabel('x');
ylabel('y');
axis equal;
