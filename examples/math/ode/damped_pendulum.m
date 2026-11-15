% Damped Nonlinear Pendulum
b = 0.15;
f = @(t, y) [y(2); -b * y(2) - sin(y(1))];
[t, y] = ode23(f, [0, 20], [3.0; 0.0]);

figure(1);
subplot(2, 1, 1);
plot(t, y(:, 1));
title('Pendulum Angle vs Time');
xlabel('Time (s)');
ylabel('Angle \theta (rad)');
grid on;

subplot(2, 1, 2);
plot(y(:, 1), y(:, 2));
title('Phase Portrait Spiral to Equilibrium');
xlabel('Angle \theta (rad)');
ylabel('Angular Velocity \omega (rad/s)');
grid on;
