% SVD Low-Rank Approximation
A = [1 2 3 4; 2 4 6 8; 3 6 9 12; 4 8 12 17];
[U, S, V] = svd(A);
sing_vals = diag(S);

figure(1);
semilogy(1:length(sing_vals), sing_vals);
title('Singular Value Spectrum Decay');
xlabel('Singular Value Index');
ylabel('Singular Value \sigma_i');
grid on;
