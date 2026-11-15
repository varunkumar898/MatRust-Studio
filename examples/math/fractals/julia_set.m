% Filled Julia Set (c = -0.7 + 0.27015i)
nx = 80; ny = 60;
x = linspace(-1.5, 1.5, nx);
y = linspace(-1.2, 1.2, ny);
J = zeros(ny, nx);
cr = -0.7; ci = 0.27015;
max_iter = 30;

for r = 1:ny
    y0 = y(r);
    for c = 1:nx
        zr = x(c);
        zi = y0;
        iter = 0;
        while (zr^2 + zi^2) <= 4.0 && iter < max_iter
            zr_new = zr^2 - zi^2 + cr;
            zi = 2.0 * zr * zi + ci;
            zr = zr_new;
            iter = iter + 1;
        end
        J(r, c) = iter;
    end
end

figure(1);
imagesc(J);
title('Filled Julia Set (c = -0.7 + 0.27015i)');
xlabel('Re(z)');
ylabel('Im(z)');
colorbar;
