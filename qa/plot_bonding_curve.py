import matplotlib.pyplot as plt
import numpy as np

def y_of_x(x, A, K, C):
    """
    Bonding curve formula
    """
    return A - K / (C + x)

def price_of_x(x, K, C):
    """
    Marginal price formula:
    """
    return (C + x)**2 / K

def main():
    # Parameters for the bonding curve
    A = 1_073_000_191.0  # Maximum number of tokens
    K = 32_190_005_730.0  # Curve "speed" parameter
    C = 30.0  # Virtual pool / offset

    x_min, x_max = 0.0, 300.0
    x_vals = np.linspace(x_min, x_max, 301)

    # Calculate y(x) and price(x)
    y_vals = [y_of_x(x, A, K, C) for x in x_vals]
    price_vals = [price_of_x(x, K, C) for x in x_vals]

    fig, ax1 = plt.subplots(figsize=(8, 6))

    # Plot y(x) — cumulative minted tokens
    color_y = "red"
    ax1.set_xlabel("x (base asset contributed)")
    ax1.set_ylabel("y(x) (cumulative minted tokens)", color=color_y)
    ax1.plot(x_vals, y_vals, color=color_y, label="Minted tokens")
    ax1.tick_params(axis='y', labelcolor=color_y)
    ax1.grid(True)

    # Plot price(x) on a secondary Y-axis
    ax2 = ax1.twinx()
    color_p = "blue"
    ax2.set_ylabel("Price", color=color_p)
    ax2.plot(x_vals, price_vals, color=color_p, label="Price")
    ax2.tick_params(axis='y', labelcolor=color_p)

    plt.title("Bonding Curve: Minted Tokens & Price")
    plt.tight_layout()
    plt.savefig("bonding_curve.png", dpi=70)
    plt.show()

if __name__ == "__main__":
    main()