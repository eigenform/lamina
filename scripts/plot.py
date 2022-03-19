#!/usr/bin/python3

import matplotlib
import matplotlib.pyplot as plt
from matplotlib.ticker import MultipleLocator
from matplotlib import colors
import numpy as np
from sys import argv

if len(argv) < 3:
    print("usage: plot.py <file> <graph title>")
    exit()

matplotlib.rcParams['font.family'] = ['monospace']

nops = []
mins = []
avgs = []
maxs = []

with open(argv[1], "r") as f:
    for line in f.readlines():
        x = line.strip().split()
        num_nops = int(x[0].rstrip(":"))
        cycles_min   = float(x[1].split("=")[1])
        cycles_avg   = float(x[2].split("=")[1])
        cycles_max   = float(x[3].split("=")[1])

        nops.append(num_nops)
        mins.append(cycles_min)
        avgs.append(cycles_avg)
        maxs.append(cycles_max)


fig, ax = plt.subplots()
ax.set_xlim(nops[0], nops[-1])
ax.xaxis.set_minor_locator(MultipleLocator(4))
ax.xaxis.set_ticks(np.arange(nops[0], nops[-1], 4))

ax.plot(nops, mins, alpha=0.75, ls=':', c='green', lw=1.0, label='min')
ax.plot(nops, avgs, c='black', lw=1.0, label='avg')
ax.plot(nops, maxs, alpha=0.75, ls=':', c='red', lw=1.0, label='max')

plt.legend()
plt.xlabel("# of instructions")
plt.ylabel("Cycles elapsed (APERF)")
plt.xticks(rotation=90)
plt.grid(True)
plt.tight_layout()
plt.title(argv[2])
plt.show()

