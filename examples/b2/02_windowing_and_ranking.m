% Example 2: Moving Windows, Cumulative Extrema, and Sorting
disp('=== Moving Windows and Cumulative Operations ===');

signal = [1.0, 2.0, 10.0, 4.0, 5.0, 6.0, 7.0];
disp('Original signal:');
disp(signal);

disp('Moving mean with window length 3:');
disp(movmean(signal, 3));

disp('Moving median with window length 3:');
disp(movmedian(signal, 3));

disp('Cumulative maximum:');
disp(cummax(signal));

disp('Cumulative minimum:');
disp(cummin(signal));

disp('Histogram counts with edges [0 3 6 12]:');
disp(histcounts(signal, [0, 3, 6, 12]));

disp('=== Matrix Sorting and Unique Elements ===');
A = [3, 20; 1, 40; 3, 10; 2, 50];
disp('Matrix A:');
disp(A);

disp('Rows sorted by col 1 asc, col 2 desc:');
disp(sortrows(A, [1, -2]));

disp('Unique elements of signal with duplicates:');
dup = [5, 2, 8, 2, 5, 1];
disp(unique(dup));
