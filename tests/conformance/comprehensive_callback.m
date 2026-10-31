% Comprehensive callback verification script
acc = 0;
for i = 1:3
    acc = acc + i;
end
s = struct();
s.val = acc;
c = {10, 20};
f = @(x) x * 2;
res_cell = cellfun(f, c);
x = [1, 2];
x(end+1) = 5;
A = [1, 2; 3, 4; 5, 6];
A(2,:) = [];
At = A';
total = s.val + sum(res_cell) + sum(x) + At(1,1) + At(2,2);
out = sprintf('Result=%d', total)
