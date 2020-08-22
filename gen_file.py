from pathlib import Path
import random

o = Path(__file__).parent / "large_out.txt"
size = (1024 ** 2) * 1

with open(o, "w") as f:
    f.write("1.")
    for i in range(0, size):
        dice1 = random.randint(0, 99)
        if dice1 == 0:
            h = chr(random.randint(0, 127))
            assert len(h) == 1
            f.write(h)
        else:
            h = hex(random.randint(0, 15))[2:]
            assert len(h) == 1
            f.write(h)

