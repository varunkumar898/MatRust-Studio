% Nonlinear Coupled System Equilibrium
f = @(x) [x(1)^2 + x(2)^2 - 4; exp(x(1)) + x(2) - 1];
x0 = [0.5; -1.5];
[x_sol, fval] = fsolve(f, x0);

disp('Equilibrium Solution x =');
disp(x_sol);
disp('Residual Norm =');
disp(norm(fval));
