% Linear Chirp Time-Frequency Spectrogram
Fs = 1000;
t = 0:1/Fs:1;
f0 = 20; f1 = 200;
chirp_signal = sin(2*pi * (f0*t + 0.5*(f1-f0)*t.^2));
[S, F, T] = spectrogram(chirp_signal, 128, 64, 128, Fs);

figure(1);
imagesc(T, F, abs(S));
title('Linear Chirp Spectrogram (20 Hz -> 200 Hz)');
xlabel('Time (s)');
ylabel('Frequency (Hz)');
colorbar;
