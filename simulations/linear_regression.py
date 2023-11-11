import pandas as pd
import numpy as np

def linear_regression(X,Y):

    N = len(X)

    x_mean = np.mean(X)
    y_mean = np.mean(Y)

    SS_xy = np.sum(Y*X) - N*y_mean*x_mean
    SS_xx = np.sum(X*X) - N*x_mean*x_mean

    b_1 = SS_xy / SS_xx
    b_0 = y_mean - b_1*x_mean

    return (b_0, b_1)