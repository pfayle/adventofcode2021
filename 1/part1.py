import sys

prev = None
m = 0
for n in sys.stdin:
  n = int(n)
  if prev == None:
    prev = n
    continue
  if n > prev:
    m += 1
  prev = n
print m
