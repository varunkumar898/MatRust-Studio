% Continuous State Transition Matrix
A = [0 1; -4 -0.5];
t = linspace(0, 10, 100);
x1 = zeros(1, length(t));
for k = 1:length(t)
    Phi = expm(A * t(k));
    x = Phi * [1; 0];
    x1(k) = x(1);
end

figure(1);
plot(t, x1);
title('Damped Transient Response via expm(At)');
xlabel('Time t (s)');
ylabel('State x_1(t)');
grid on;
