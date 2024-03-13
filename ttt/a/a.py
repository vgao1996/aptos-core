import numpy as np
from scipy.optimize import nnls, least_squares

# Define matrices A and B
A = np.array([[122, 360000],
              [189, 572000],
              [352, 873000],
              [220, 433000],
              [64, 202000],
              [66, 87000]])

B = np.array([287.61238, 476.7069, 847.02758, 494.35278, 153.83918, 102.76214])

# Find non-negative least squares solution
x, _ = nnls(A, B)

# Print solution
print("non-negative x and y:")
print(x)

res_lsq = least_squares(lambda x: A.dot(x) - B, np.zeros(2))
x_lsq = res_lsq.x
print("least squares solution:")
print(x_lsq)