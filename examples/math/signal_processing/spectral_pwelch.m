% Welch Power Spectral Density Estimation
Fs = 1000;
t = 0:1/Fs:2;
x = cos(2*pi*100*t) + 0.5*sin(2*pi*250*t) + 0.3*randn(size(t));
[pxx, f] = pwelch(x, 256, 128, 256, Fs);

figure(1);
semilogy(f, pxx);
title('Welch Power Spectral Density Estimate');
xlabel('Frequency (Hz)');
ylabel('Power / Frequency (dB/Hz)');
grid on;
