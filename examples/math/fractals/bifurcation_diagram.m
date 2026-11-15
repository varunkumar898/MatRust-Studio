% Logistic Map Period-Doubling Bifurcation
r_vals = linspace(2.5, 4.0, 300);
r_plot = [];
x_plot = [];

for k = 1:length(r_vals)
    r = r_vals(k);
    x = 0.5;
    for i = 1:150
        x = r * x * (1 - x);
    end
    for i = 1:50
        x = r * x * (1 - x);
        r_plot = [r_plot, r];
        x_plot = [x_plot, x];
    end
end

figure(1);
scatter(r_plot, x_plot);
title('Logistic Map Bifurcation: Period-Doubling to Chaos');
xlabel('Growth Parameter r');
ylabel('Attractor State x^*');
