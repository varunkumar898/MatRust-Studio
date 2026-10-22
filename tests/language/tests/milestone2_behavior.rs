// Verified behavior tests — MatRust Studio interpreter (milestone 2)
// Run via `matlabr` or through desktop execution.

// 1. Range semantics
// total = 0; for i = 1:5; total = total + i; end  => 15

// 2. While loop
// i = 1; while i <= 5; i = i + 1; end => 6

// 3. Continue / nested control
// total = 0; for i = 1:4; if i == 3; continue; end; total = total + i; end => 7

// 4. Negative step / empty / 1-based ranges
// 5:-1:1 => [5 4 3 2 1]; 1:0 => []

// 5. Indexing with `end` and `:` (partial — `A(1,:)` works; `end` in expression needs array context)

// 6. If / elseif / else / end + while / for / break / continue / switch / case / otherwise

// 7. User-defined functions / complex / logical truth rules: partial
