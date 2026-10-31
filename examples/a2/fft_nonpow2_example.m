% Example: Non-Power-of-2 Fast Fourier Transform
% Demonstrates Bluestein / mixed-radix FFT via rustfft on composite and prime lengths

disp('=== FFT on Non-Power-of-2 Lengths ===');

% Prime length: N = 7
disp('1. Prime length N = 7');
x_prime = [1.0; 2.0; 3.0; 4.0; 5.0; 6.0; 7.0];
Y_prime = fft(x_prime);
x_prime_rec = ifft(Y_prime);
disp('Roundtrip error ||real(ifft(fft(x))) - x||:');
err_prime = norm(real(x_prime_rec) - x_prime);
disp(err_prime);

% Composite non-power-of-2: N = 1000
disp('2. Composite length N = 1000');
x_comp = (1:1000)';
Y_comp = fft(x_comp);
x_comp_rec = ifft(Y_comp);
disp('Roundtrip error for N=1000:');
err_comp = norm(real(x_comp_rec) - x_comp);
disp(err_comp);
