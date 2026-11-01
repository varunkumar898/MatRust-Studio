x = [1 2 3 4];
y = ifft(fft(x));
res = round(real(y));