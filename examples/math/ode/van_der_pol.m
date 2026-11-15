% Van der Pol Relaxation Oscillator
mu = 1.0;
f = @(t, y) [y(2); mu * (1 - y(1)^2) * y(2) - y(1)];
[t, y] = ode45(f, [0, 20], [2; 0]);

figure(1);
subplot(2, 1, 1);
plot(t, y(:, 1));
title('Van der Pol Oscillator: Position vs Time');
xlabel('Time t (s)');
ylabel('Position y(1)');
grid on;

subplot(2, 1, 2);
plot(y(:, 1), y(:, 2));
title('Phase Space Limit Cycle');
xlabel('Position y(1)');
ylabel('Velocity y(2)');
grid on;
