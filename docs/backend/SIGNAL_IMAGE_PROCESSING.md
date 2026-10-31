# MatRust Studio Signal and Image Processing

## 1. Overview

MatRust Studio includes native transforms, discrete-time digital filtering, window functions, and image manipulation routines.

## 2. Signal Processing Capabilities

### 2.1 Spectral Transforms
- `fft(x)`: 1-D Fast Fourier Transform computed using the O(N log N) `rustfft` backend.
- `ifft(X)`: 1-D Inverse Fast Fourier Transform.
- `fft2(A)` / `ifft2(A)`: 2-D Fast Fourier Transforms.
- `fftshift(X)`: Shifts zero-frequency component to the center of the spectrum.

### 2.2 Digital Filtering
- `filter(b, a, x)`: Direct Form II Transposed digital filter implementation. Supports arbitrary IIR and FIR filters.
- `filtfilt(b, a, x)`: Zero-phase digital filtering via forward-backward filtering passes.
- `xcorr(x, y)` / `xcorr(x)`: Cross-correlation and auto-correlation via discrete convolution.

### 2.3 Window Functions
- `hamming(N)`: Symmetric Hamming window.
- `hann(N)`: Symmetric Hann window.
- `blackman(N)`: Symmetric Blackman window.

## 3. Image Processing Capabilities

- `rgb2gray(RGB)`: Converts truecolor RGB images ($H \times W \times 3$) to 2-D grayscale arrays ($H \times W$) using ITU-R BT.601 luminance weights ($0.2989 R + 0.5870 G + 0.1140 B$). Supports both `Double` and `UInt8` representations.
- `imresize(I, scale)`: Bilinear/nearest-neighbor 2-D spatial array interpolation for arbitrary scaling factors.
- `imfilter(I, h)`: 2-D spatial correlation filtering with arbitrary kernels.
