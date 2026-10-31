% Example 3: Delimited Matrix I/O (readmatrix and writematrix)
disp('=== Delimited Matrix I/O ===');

M = [1.5, 2.75, 3.125; 4.25, 5.5, 6.875; 7.0, 8.125, 9.5];
disp('Original numerical matrix M:');
disp(M);

filename = 'target/temp_demo_matrix.csv';
writematrix(M, filename);
disp(['Written matrix to ', filename]);

disp('Reading back matrix from CSV:');
M_read = readmatrix(filename);
disp(M_read);

disp('Roundtrip difference max abs:');
diff = M - M_read;
disp(max(abs(diff(:))));
