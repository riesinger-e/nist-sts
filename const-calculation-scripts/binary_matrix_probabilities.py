"""
Calculate the 3 probabilities for the Binary Matrix Rank Test according to section 3.5 - 4 decimal places are not very precise.
"""

M = 32
Q = 32

# calculate upper bound m
m = min(M, Q)

for r in range(m-2, m+1):
    p_r = pow(2, r * (Q + M - r) - M*Q)

    for i in range(r):
        part1 = ((1 - pow(2, i - Q)) * (1 - pow(2, i - M)))
        part2 = (1 - pow(2, i - r))
        p_r *= part1 / part2

    print(f"p_{r} = {str(p_r)}")
