% Rosenbrock Banana Valley Minimization via fminunc
rosenbrock = @(x) 100 * (x(2) - x(1)^2)^2 + (1 - x(1))^2;
x0 = [-1.2; 1.0];
[x_opt, fval] = fminunc(rosenbrock, x0);

disp('Optimal Solution x* =');
disp(x_opt);
disp('Minimum Function Value f(x*) =');
disp(fval);

% Contour map visualization
[X, Y] = meshgrid(-1.5:0.05:1.5, -0.5:0.05:1.5);
Z = 100 * (Y - X.^2).^2 + (1 - X).^2;
figure(1);
contour(X, Y, Z);
hold on;
plot(x_opt(1), x_opt(2));
plot(x0(1), x0(2));
hold off;
title('Rosenbrock Valley Minimization');
xlabel('x_1');
ylabel('x_2');
legend('Contour Levels', 'Optimal Minimum (1, 1)', 'Initial Guess (-1.2, 1.0)');
