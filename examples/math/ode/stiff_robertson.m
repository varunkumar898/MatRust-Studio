% Stiff Robertson Chemical Kinetics
f = @(t, y) [-0.04*y(1) + 1e4*y(2)*y(3); 0.04*y(1) - 1e4*y(2)*y(3) - 3e7*y(2)^2; 3e7*y(2)^2];
[t, y] = ode15s(f, [0, 100], [1; 0; 0]);

figure(1);
semilogx(t, y(:, 1));
hold on;
semilogx(t, y(:, 2) * 1e4);
semilogx(t, y(:, 3));
hold off;
title('Robertson Reaction Kinetics (ode15s)');
xlabel('Time t (s)');
ylabel('Fractional Mass');
legend('Species A (y1)', 'Species B (y2 * 1e4)', 'Species C (y3)');
grid on;
