% Exponential Decay Curve Fit
t_data = [0.0; 0.5; 1.0; 1.5; 2.0; 3.0; 4.0; 5.0];
y_data = [5.12; 3.25; 2.11; 1.48; 1.15; 0.72; 0.58; 0.51];
model = @(p, t) p(1) * exp(-p(2) * t) + p(3);

p0 = [4.0; 1.0; 0.2];
p_opt = lsqcurvefit(model, p0, t_data, y_data);

t_fine = linspace(0, 5, 100);
y_fit = model(p_opt, t_fine);

figure(1);
plot(t_fine, y_fit);
hold on;
plot(t_data, y_data);
hold off;
title('Nonlinear Exponential Decay Fit (lsqcurvefit)');
xlabel('Time t');
ylabel('Response y(t)');
legend('Model Fit', 'Data Samples');
grid on;
