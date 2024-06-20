"""
Calculate the probabilities pi_i for the "Test for the Longest Run of Ones in a Block", according to section 3.4.
This can take a very long time.
"""

# comb is the binomial coefficient function for integers (when used with exact=true)
from scipy.special import comb
import math
from multiprocessing import Pool

# maximum of processes to spawn - 7 means that the most time-consuming probabilities can be calculated completely in parallel.
PROCESSES = 7

# the classes to calculate per K and M
CLASSES = {
    (3, 8): [1, 2, 3, 4],
    (5, 128): [4, 5, 6, 7, 8, 9],
    (6, 10_000): [10, 11, 12, 13, 14, 15, 16],
}


def binomial(n: int, k: int) -> int:
    """
    The binomial coefficient for integers, exact. n choose k
    :param n:
    :param k:
    :return: the binomial coefficient
    """
    return comb(n, k, exact=True)


def calculate_p_m(M: int, m: int) -> float:
    """
    Calculate P(v <= m) = sum_{r = 0}^{M} ( binomial(M, r) * P(v <= m | r) / (2^M) )
    :param M: M, as seen in the formula
    :param m: m, as seen in the formula
    :return: the probability
    """

    p_sum = 0.0
    for r in range(M+1):
        print(f"current iteration: {r} / {M+1} for class {m}")
        # Calculate P(v <= m | r) = 1 / binomial(M, r) * sum_{j = 0}^{U} ( (-1)^j * binomial(M-r+1, j) * binomial(M-j*(m+1), M-r) )
        # but don't do the 1 / binomial(M, r) division - in the formula for P(v <= m), a multiplication by binomial(M, r) would be done that now doesn't need to be done

        p_sum_r = 0
        # calculate U
        U = min(M - r + 1, math.floor(r / (m + 1)))
        for j in range(U+1):
            p_sum_r += pow(-1, j) * binomial(M - r + 1, j) * binomial(M - j * (m + 1), M - r)

        # p_sum_r is now sum_{j = 0}^{U} ( (-1)^j * binomial(M-r+1, j) * binomial(M-j*(m+1), M-r) ) = P(v <= m | r) * binomial(M, r)

        p_sum += p_sum_r / pow(2, M)

    return p_sum


def main():
    # store probabilities
    all_probabilities = dict()

    # calculate for each given combination of K and M
    for K, M in [(3, 8), (5, 128), (6, 10_000)]:
        """
        Relevant formulas:
        U = min(M - r + 1, floor(r / (m + 1)) )
        P(v <= m | r) = 1 / binomial(M, r) * sum_{j = 0}^{U} ( (-1)^j * binomial(M-r+1, j) * binomial(M-j*(m+1), M-r) )
        P(v <= m) = sum_{r = 0}^{M} ( binomial(M, r) * P(v <= m | r) / (2^M) )
        """
        # from 3.4.: P(v <= m | r) = 1 / binomial(M, r) * sum_{j = 0}^{U}

        # calculate first class: this is the base for all following calculations
        classes = CLASSES[(K, M)]

        args = [(M, c) for c in classes]

        # parallelized
        with Pool(PROCESSES) as p:
            probabilities = p.starmap(calculate_p_m, args)

        for i in range(1, len(probabilities) - 1):
            probabilities[i] -= sum(probabilities[:i])

        # last probability is a special case
        probabilities[-1] = 1.0 - sum(probabilities[:-1])

        all_probabilities[(K, M)] = probabilities

    print()

    for ((K, M), probabilities) in all_probabilities.items():
        print(f"K = {K}, M = {M}")
        for (idx, prob) in enumerate(probabilities):
            # print with full precision by forcing str()
            print(f"pi_{idx} = {str(prob)}")
        print()


if __name__ == "__main__":
    print("Have you stopped everything turning off your system automatically?")
    main()
    print("Remember to restart any stopped services!")
