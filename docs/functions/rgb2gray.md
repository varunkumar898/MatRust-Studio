# `rgb2gray`

Convert RGB image to grayscale.

## Syntax

```matlab
I = rgb2gray(RGB)
```

## Description

`I = rgb2gray(RGB)` converts the truecolor image array `RGB` ($H \times W \times 3$) to a 2-D grayscale intensity image `I` using the ITU-R standard luminance equation:
$$I = 0.2989 R + 0.5870 G + 0.1140 B$$
