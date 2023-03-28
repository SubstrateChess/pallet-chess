import json
import matplotlib.pyplot as plt

f = open('benchmarks.json')
benchmarks = json.load(f)
f.close()

make_move_benchmarked_time_results = benchmarks[0]['time_results']

t_array = []
i_array = []

for entry in make_move_benchmarked_time_results:
    t_array.append(entry['extrinsic_time'])
    i_array.append(entry['components'][0][1])

t_avg = sum(t_array) / len(t_array)

plt.title('Histogram')
plt.hist(t_array, bins=500, range=(0,t_avg*5))
plt.axvline(t_avg, color='k', linestyle='dashed', linewidth=1, label='Average: {:.2f} ns'.format(t_avg))
plt.axvline(t_avg*3, color='r', linestyle='dashed', linewidth=1, label='3*Average: {:.2f} ns'.format(3*t_avg))
plt.xlabel('execution time (ns)')
plt.legend()
plt.savefig('benchmarks_histogram.png')

plt.figure().clear()

plt.title('Plot (i vs execution time)')
plt.plot(i_array, t_array, '.')
plt.axhline(t_avg, color='k', linestyle='dashed', linewidth=1, label='Average: {:.2f} ns'.format(t_avg))
plt.axhline(t_avg*3, color='r', linestyle='dashed', linewidth=1, label='3*Average: {:.2f} ns'.format(3*t_avg))
plt.ylabel('execution time (ns)')
plt.xlabel('i')
plt.legend()
plt.savefig('benchmarks_plot.png')
