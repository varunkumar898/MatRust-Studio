% Linear Programming Production Optimization
% Maximize 3*x1 + 5*x2  <=> Minimize -3*x1 - 5*x2
f = [-3; -5];
A = [1, 0; 0, 2; 3, 2];
b = [4; 12; 18];
lb = [0; 0];
[x_opt, fval] = linprog(f, A, b, [], [], lb, []);

disp('Optimal Production Levels [x1; x2] =');
disp(x_opt);
disp('Maximum Profit =');
disp(-fval);
