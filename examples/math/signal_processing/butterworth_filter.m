% Butterworth Low-pass Filter Frequency Response
% 2nd-order digital Butterworth filter coefficients:
b = [0.067455, 0.13491, 0.067455];
a = [1.0, -1.14298, 0.41280];
[h, w] = freqz(b, a, 512);

mag_db = 20 * log10(abs(h) + 1e-12);
figure(1);
subplot(2, 1, 1);
plot(w / pi, mag_db);
title('Butterworth Filter Magnitude Response');
xlabel('Normalized Frequency (\times \pi rad/sample)');
ylabel('Magnitude (dB)');
grid on;

subplot(2, 1, 2);
plot(w / pi, angle(h));
title('Phase Response');
xlabel('Normalized Frequency (\times \pi rad/sample)');
ylabel('Phase (rad)');
grid on;
