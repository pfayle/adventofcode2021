import sys

def net_zeroes(ls):
    r = []
    for w in ls:
        for i in range(len(w)):
            if i >= len(r):
                r.append(0)
            if w[i] == "0":
                r[i] += 1
            else:
                r[i] -= 1
    return r

def reduce_list(ls, net, success_fn):
    r = []
    for w in ls:
        if success_fn(w, net):
            r.append(w)
    return r

# oxy: most common (direction 1) or 1 if equal
# co2: least common (direction -1) or 0 if equal
def success(w, net, pos, direction, default):
    agg = net[pos]
    c = int(w[pos])
    if agg == 0:
        return c == default
    if agg * direction > 0:
        return c == 0
    else:
        return c == 1

whole_list = []
for w in sys.stdin:
    whole_list.append(w[:-1])
    length = len(w)-1

oxy_list = co2_list = whole_list
oxy_success_fn = lambda w, net: success(w, net, i, 1, 1)
co2_success_fn = lambda w, net: success(w, net, i, -1, 0)
for i in range(length):
    if len(oxy_list) > 1:
        oxy_zeroes = net_zeroes(oxy_list)
        oxy_list = reduce_list(oxy_list, oxy_zeroes, oxy_success_fn)
    if len(co2_list) > 1:
        co2_zeroes = net_zeroes(co2_list)
        co2_list = reduce_list(co2_list, co2_zeroes, co2_success_fn)

oxy_final = int(oxy_list[0],2)
co2_final = int(co2_list[0],2)
print(oxy_final*co2_final)
