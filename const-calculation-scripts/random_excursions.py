"""
Calculate the probabilities for the random excursion test, as described on page 3-23.
"""


def pi_0(x):
    return 1 - 1 / (2 * abs(x))


def pi_k(x, k):
    return 1 / (4*x*x) * pow(1 - 1 / (2 * abs(x)), k - 1)


def pi_5(x):
    return 1 / (2 * abs(x)) * pow(1 - 1 / (2 * abs(x)), 4)


def pi(x, k):
    if k == 0:
        return pi_0(x)
    elif k == 5:
        return pi_5(x)
    else:
        return pi_k(x, k)


print("[")
for k in range(0, 6):
    # all calculations use absolute x values - don't need to calculate the negative ones, just print them in reverse order
    pi_values = [pi(x, k) for x in range(5) if x != 0]

    print("\t[ ", end="")
    for p in reversed(pi_values):
        print(f"{p}, ", end="")
    for p in pi_values:
        print(f"{p}, ", end="")

    print("]")
print("]")

# used as a basis to calculate fractions (manually)

