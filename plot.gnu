set datafile separator ";"
set xlabel "X"
set ylabel "Y"
set terminal wxt size 800,600
plot "data.csv" using 1:2 with linespoints title "Plot"