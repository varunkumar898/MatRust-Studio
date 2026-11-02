f = @(x) x.^2 + 2.*x + 1;
y = f(4);
v = [1 2 3];
res = arrayfun(@(x) x*2, v);
