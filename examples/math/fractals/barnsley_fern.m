% Barnsley Fern Iterated Function System
N = 5000;
x = zeros(1, N);
y = zeros(1, N);
x_curr = 0.0; y_curr = 0.0;
p = rand(1, N);

for i = 1:N
    prob = p(i);
    if prob < 0.01
        x_next = 0.0;
        y_next = 0.16 * y_curr;
    elseif prob < 0.86
        x_next = 0.85 * x_curr + 0.04 * y_curr;
        y_next = -0.04 * x_curr + 0.85 * y_curr + 1.6;
    elseif prob < 0.93
        x_next = 0.2 * x_curr - 0.26 * y_curr;
        y_next = 0.23 * x_curr + 0.22 * y_curr + 1.6;
    else
        x_next = -0.15 * x_curr + 0.28 * y_curr;
        y_next = 0.26 * x_curr + 0.24 * y_curr + 0.44;
    end
    x_curr = x_next;
    y_curr = y_next;
    x(i) = x_curr;
    y(i) = y_curr;
end

figure(1);
scatter(x, y);
title('Barnsley Fern Fractal Attractor');
xlabel('x');
ylabel('y');
