a = [0xFF] * 256

for i in range(0, 10):
    a[ord("0") + i] = i

for i in range(0, 6):
    a[ord("a") + i] = 10 + i

for i in range(0, 6):
    a[ord("A") + i] = 10 + i

print(a)
