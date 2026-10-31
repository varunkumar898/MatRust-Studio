# MatRust Studio Machine Learning, Robotics, and Embedded Systems

## 1. Machine Learning Capabilities

### 1.1 Supervised Learning: Linear Regression
- `model = fitlm(X, y)`: Fits an ordinary least squares linear regression model:
  $$y = \beta_0 + \beta_1 X_1 + \dots + \beta_p X_p + \epsilon$$
  Computes parameter estimates $\beta = (X_{aug}^T X_{aug})^{-1} X_{aug}^T y$, $R^2$ score, root mean squared error (RMSE), and residual vector.
- `y_hat = predict(model, X)`: Computes predicted responses using estimated regression coefficients.

### 1.2 Unsupervised Learning: K-Means Clustering
- `[idx, C] = kmeans(X, k)`: Partitions $N$ observations into $k$ clusters using deterministic, seeded Lloyd's algorithm. Returns cluster assignment vector `idx` and centroid matrix `C`.

### 1.3 Classification Evaluation
- `C = confusionmat(group, grouphat)`: Computes the classification confusion matrix measuring counts of true versus predicted classes.

## 2. Robotics & Spatial Mathematics

### 2.1 3-D Rotations & Homogeneous Transformations
- `rotx(theta)` / `roty(theta)` / `rotz(theta)`: 3x3 orthonormal rotation matrices about principal Cartesian axes.
- `trotx(theta)` / `troty(theta)` / `trotz(theta)`: 4x4 homogeneous transformation matrices for pure rotations.
- `transl(x, y, z)`: 4x4 homogeneous translation matrix.

### 2.2 Kinematics
- `[x, y] = fkine_2link(l1, l2, q1, q2)`: Analytical forward kinematics for 2-link planar manipulator arm.
- `[q1, q2] = ikine_2link(l1, l2, x, y)`: Analytical inverse kinematics for 2-link planar arm (elbow-up configuration). Rejects unreachable targets outside link workspace with clear diagnostic error.

### 2.3 Trajectory Generation
- `q = jtraj(q0, q1, n)`: Generates smooth quintic polynomial joint trajectory from initial configuration $q_0$ to final configuration $q_1$ over $n$ steps with zero boundary velocities and accelerations.

## 3. Hardware & Embedded Systems

- Statically typed, MISRA-friendly C99 code generation architecture designed for microcontrollers (ESP32, STM32) and embedded execution without dynamic heap allocation.
- Clean trait-based device abstractions allowing hardware testing and simulation without requiring physical USB/serial devices attached.
