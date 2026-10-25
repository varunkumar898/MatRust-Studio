% RustLab Milestone A3: Numeric Types and Saturating Arithmetic Example
disp('=== RustLab A3: Numeric Types Demonstration ===');

% 1. Saturating Integer Arithmetic
u = uint8(250);
disp('Initial uint8 value:');
disp(u);

u_sat = u + uint8(20);
disp('Saturated addition (uint8(250) + uint8(20) = 255):');
disp(u_sat);

u_zero = uint8(3) - uint8(10);
disp('Saturated underflow (uint8(3) - uint8(10) = 0):');
disp(u_zero);

i_max = int8(127) + 5;
disp('Signed saturated addition (int8(127) + 5 = 127):');
disp(i_max);

% 2. Round half away from zero conversions
r_pos = int32(2.5);
r_neg = int32(-2.5);
disp('Round half away from zero: int32(2.5) and int32(-2.5):');
disp([r_pos, r_neg]);

% 3. Integer Division
q_round = int32(7) / int32(2);
disp('Integer division rounded to nearest (7 / 2 = 4):');
disp(q_round);

q_fix = idivide(int32(7), int32(2), 'fix');
q_ceil = idivide(int32(7), int32(2), 'ceil');
disp('idivide fix and ceil:');
disp([q_fix, q_ceil]);

% 4. Introspection and Bitwise
disp('Class of int64(100):');
disp(class(int64(100)));

shifted = bitshift(uint8(1), 4);
disp('bitshift(uint8(1), 4):');
disp(shifted);
