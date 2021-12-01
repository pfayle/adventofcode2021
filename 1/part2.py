import sys

m = 0
res = []
for n in sys.stdin:
  res.append(int(n))
  if len(res) != 4:
    continue
  if sum(res[0:3]) < sum(res[1:4]):
    m += 1
  res = res[1:]
print m
