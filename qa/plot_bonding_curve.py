import matplotlib.pyplot as plt
import numpy as np

def y_of_x(x, A, K, C):
    """
    Bonding curve formula:
    y(x) = A - K / (C + x)
    """
    return A - K / (C + x)

def main():
    A = 1_073_000_191.0  # Maximum number of tokens
    K = 32_190_005_730.0  # Curve "speed" parameter
    C = 30.0  # Virtual pool / offset

    x_min, x_max = 0.0, 300.0
    x_vals = np.linspace(x_min, x_max, 301)
    y_vals = [y_of_x(x, A, K, C) for x in x_vals]

    plt.figure(figsize=(8, 6))
    plt.plot(x_vals, y_vals, label="y(x) = A - K / (C + x)", color="red")
    plt.title("Bonding Curve")
    plt.xlabel("x (base asset contributed)")
    plt.ylabel("y(x) (cumulative minted tokens)")
    plt.grid(True)
    plt.legend()
    plt.tight_layout()

    plt.savefig("bonding_curve.png", dpi=300)
    plt.show()

if __name__ == "__main__":
    main()