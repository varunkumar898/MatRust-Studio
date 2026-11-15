% Mandelbrot Escape-Time Fractal
nx = 80; ny = 60;
x = linspace(-2.1, 0.6, nx);
y = linspace(-1.2, 1.2, ny);
M = zeros(ny, nx);
max_iter = 30;

for r = 1:ny
    ci = y(r);
    for c = 1:nx
        cr = x(c);
        zr = 0.0;
        zi = 0.0;
        iter = 0;
        while (zr^2 + zi^2) <= 4.0 && iter < max_iter
            zr_new = zr^2 - zi^2 + cr;
            zi = 2.0 * zr * zi + ci;
            zr = zr_new;
            iter = iter + 1;
        end
        M(r, c) = iter;
    end
end

figure(1);
imagesc(M);
title('Mandelbrot Fractal Set (Escape Time)');
xlabel('Re(c)');
ylabel('Im(c)');
colorbar;
