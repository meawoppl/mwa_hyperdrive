## Correctness

| workload | output | result |
|---|---|---|
| birli-legacy | flags_01.mwaf | byte-identical |
| birli-legacy | flags_02.mwaf | byte-identical |
| birli-legacy | flags_03.mwaf | byte-identical |
| birli-legacy | flags_04.mwaf | byte-identical |
| birli-legacy | flags_05.mwaf | byte-identical |
| birli-legacy | flags_06.mwaf | byte-identical |
| birli-legacy | flags_07.mwaf | byte-identical |
| birli-legacy | flags_08.mwaf | byte-identical |
| birli-legacy | flags_09.mwaf | byte-identical |
| birli-legacy | flags_10.mwaf | byte-identical |
| birli-legacy | flags_11.mwaf | byte-identical |
| birli-legacy | flags_12.mwaf | byte-identical |
| birli-legacy | flags_13.mwaf | byte-identical |
| birli-legacy | flags_14.mwaf | byte-identical |
| birli-legacy | flags_15.mwaf | byte-identical |
| birli-legacy | flags_16.mwaf | byte-identical |
| birli-legacy | flags_17.mwaf | byte-identical |
| birli-legacy | flags_18.mwaf | byte-identical |
| birli-legacy | flags_19.mwaf | byte-identical |
| birli-legacy | flags_20.mwaf | byte-identical |
| birli-legacy | flags_21.mwaf | byte-identical |
| birli-legacy | flags_22.mwaf | byte-identical |
| birli-legacy | flags_23.mwaf | byte-identical |
| birli-legacy | flags_24.mwaf | byte-identical |
| birli-legacy | out_ch103-104.uvfits | byte-identical |
| birli-legacy | out_ch113-114.uvfits | byte-identical |
| birli-legacy | out_ch125-126.uvfits | byte-identical |
| birli-legacy | out_ch139-140.uvfits | byte-identical |
| birli-legacy | out_ch153-154.uvfits | byte-identical |
| birli-legacy | out_ch169-170.uvfits | byte-identical |
| birli-legacy | out_ch187-188.uvfits | byte-identical |
| birli-legacy | out_ch62-63.uvfits | byte-identical |
| birli-legacy | out_ch69-70.uvfits | byte-identical |
| birli-legacy | out_ch76-77.uvfits | byte-identical |
| birli-legacy | out_ch84-85.uvfits | byte-identical |
| birli-legacy | out_ch93-94.uvfits | byte-identical |
| birli-legacy-chunked | flags_01.mwaf | byte-identical |
| birli-legacy-chunked | flags_02.mwaf | byte-identical |
| birli-legacy-chunked | flags_03.mwaf | byte-identical |
| birli-legacy-chunked | flags_04.mwaf | byte-identical |
| birli-legacy-chunked | flags_05.mwaf | byte-identical |
| birli-legacy-chunked | flags_06.mwaf | byte-identical |
| birli-legacy-chunked | flags_07.mwaf | byte-identical |
| birli-legacy-chunked | flags_08.mwaf | byte-identical |
| birli-legacy-chunked | flags_09.mwaf | byte-identical |
| birli-legacy-chunked | flags_10.mwaf | byte-identical |
| birli-legacy-chunked | flags_11.mwaf | byte-identical |
| birli-legacy-chunked | flags_12.mwaf | byte-identical |
| birli-legacy-chunked | flags_13.mwaf | byte-identical |
| birli-legacy-chunked | flags_14.mwaf | byte-identical |
| birli-legacy-chunked | flags_15.mwaf | byte-identical |
| birli-legacy-chunked | flags_16.mwaf | byte-identical |
| birli-legacy-chunked | flags_17.mwaf | byte-identical |
| birli-legacy-chunked | flags_18.mwaf | byte-identical |
| birli-legacy-chunked | flags_19.mwaf | byte-identical |
| birli-legacy-chunked | flags_20.mwaf | byte-identical |
| birli-legacy-chunked | flags_21.mwaf | byte-identical |
| birli-legacy-chunked | flags_22.mwaf | byte-identical |
| birli-legacy-chunked | flags_23.mwaf | byte-identical |
| birli-legacy-chunked | flags_24.mwaf | byte-identical |
| birli-legacy-chunked | out_ch103-104.uvfits | byte-identical |
| birli-legacy-chunked | out_ch113-114.uvfits | byte-identical |
| birli-legacy-chunked | out_ch125-126.uvfits | byte-identical |
| birli-legacy-chunked | out_ch139-140.uvfits | byte-identical |
| birli-legacy-chunked | out_ch153-154.uvfits | byte-identical |
| birli-legacy-chunked | out_ch169-170.uvfits | byte-identical |
| birli-legacy-chunked | out_ch187-188.uvfits | byte-identical |
| birli-legacy-chunked | out_ch62-63.uvfits | byte-identical |
| birli-legacy-chunked | out_ch69-70.uvfits | byte-identical |
| birli-legacy-chunked | out_ch76-77.uvfits | byte-identical |
| birli-legacy-chunked | out_ch84-85.uvfits | byte-identical |
| birli-legacy-chunked | out_ch93-94.uvfits | byte-identical |
| birli-mwax | flags_117.mwaf | byte-identical |
| birli-mwax | flags_118.mwaf | byte-identical |
| birli-mwax | out.uvfits | byte-identical |
| hd-di-cal | sols.fits | byte-identical |
| hd-sols-apply | out.uvfits | byte-identical |
| hd-srclist-convert | srclist.fits | byte-identical |
| hd-vis-convert | out.uvfits | byte-identical |
| mwalib-legacy | dump.txt | byte-identical |
| mwalib-mwax | dump.txt | byte-identical |

## Time and memory (cold page cache, median of runs)

| workload | backend | wall s | user s | sys s | peak RSS MB |
|---|---|---|---|---|---|
| birli-legacy | cfitsio | 3.11 | 2.15 | 1.82 | 91 |
| birli-legacy | pure | 2.94 | 2.15 | 1.71 | 110 |
| birli-legacy-chunked | cfitsio | 2.81 | 2.06 | 1.33 | 88 |
| birli-legacy-chunked | pure | 2.72 | 2.25 | 1.19 | 85 |
| birli-mwax | cfitsio | 0.03 | 0.02 | 0.05 | 19 |
| birli-mwax | pure | 0.02 | 0.02 | 0.05 | 12 |
| hd-di-cal | cfitsio | 1.10 | 4.22 | 0.67 | 66 |
| hd-di-cal | pure | 1.08 | 4.34 | 0.70 | 60 |
| hd-sols-apply | cfitsio | 0.38 | 0.32 | 0.14 | 56 |
| hd-sols-apply | pure | 0.36 | 0.26 | 0.13 | 50 |
| hd-srclist-convert | cfitsio | 0.26 | 0.01 | 0.01 | 20 |
| hd-srclist-convert | pure | 0.26 | 0.00 | 0.00 | 13 |
| hd-vis-convert | cfitsio | 3.46 | 0.84 | 1.59 | 664 |
| hd-vis-convert | pure | 2.62 | 0.82 | 1.39 | 658 |
| mwalib-legacy | cfitsio | 2.14 | 1.55 | 0.47 | 29 |
| mwalib-legacy | pure | 2.05 | 1.42 | 0.43 | 23 |
| mwalib-mwax | cfitsio | 0.55 | 0.36 | 0.18 | 78 |
| mwalib-mwax | pure | 0.69 | 0.40 | 0.27 | 108 |

## Time and memory (warm page cache, median of runs)

| workload | backend | wall s | user s | sys s | peak RSS MB |
|---|---|---|---|---|---|
| birli-legacy | cfitsio | 2.75 | 2.20 | 1.60 | 91 |
| birli-legacy | pure | 2.50 | 2.13 | 1.47 | 87 |
| birli-legacy-chunked | cfitsio | 2.79 | 2.48 | 1.05 | 88 |
| birli-legacy-chunked | pure | 2.32 | 2.25 | 0.92 | 85 |
| birli-mwax | cfitsio | 0.04 | 0.02 | 0.05 | 19 |
| birli-mwax | pure | 0.02 | 0.01 | 0.05 | 12 |
| hd-di-cal | cfitsio | 1.08 | 4.30 | 0.66 | 65 |
| hd-di-cal | pure | 1.08 | 4.08 | 0.68 | 60 |
| hd-sols-apply | cfitsio | 0.37 | 0.40 | 0.16 | 55 |
| hd-sols-apply | pure | 0.36 | 0.24 | 0.14 | 50 |
| hd-srclist-convert | cfitsio | 0.26 | 0.01 | 0.01 | 20 |
| hd-srclist-convert | pure | 0.26 | 0.00 | 0.00 | 13 |
| hd-vis-convert | cfitsio | 1.75 | 0.83 | 1.15 | 664 |
| hd-vis-convert | pure | 1.70 | 0.87 | 1.11 | 659 |
| mwalib-legacy | cfitsio | 1.70 | 1.45 | 0.25 | 30 |
| mwalib-legacy | pure | 1.53 | 1.31 | 0.22 | 23 |
| mwalib-mwax | cfitsio | 0.53 | 0.36 | 0.16 | 78 |
| mwalib-mwax | pure | 0.67 | 0.42 | 0.25 | 108 |

## I/O syscalls on the workload's FITS files

| workload | backend | reads | read bytes | reads/MB | read size min/median/max | writes | write bytes | writes/MB | write size min/median/max | lseek | openat | mmap | fsync+fdatasync | rename |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| birli-legacy | cfitsio | 47977 | 460.7 MiB | 99.32 | 1.1 KiB / 4.0 KiB / 28.0 KiB | 163152 | 611.0 MiB | 254.65 | 59 B / 4.0 KiB / 20.9 KiB | 33463 | 460 | 0 | 0 | 0 |
| birli-legacy | pure | 965 | 407.0 MiB | 2.26 | 2 B / 1.0 MiB / 1.0 MiB | 720 | 594.6 MiB | 1.15 | 61 B / 1.0 MiB / 1.0 MiB | 0 | 302 | 0 | 0 | 72 |
| birli-legacy-chunked | cfitsio | 48193 | 461.4 MiB | 99.60 | 1.1 KiB / 4.0 KiB / 28.0 KiB | 163273 | 611.1 MiB | 254.78 | 61 B / 4.0 KiB / 21.0 KiB | 33679 | 508 | 0 | 0 | 0 |
| birli-legacy-chunked | pure | 1013 | 408.0 MiB | 2.37 | 2 B / 1.0 MiB / 1.0 MiB | 769 | 595.6 MiB | 1.23 | 60 B / 1.0 MiB / 1.0 MiB | 0 | 350 | 0 | 0 | 96 |
| birli-mwax | cfitsio | 156 | 535.9 KiB | 284.26 | 448 B / 4.0 KiB / 4.0 KiB | 38 | 75.5 KiB | 491.33 | 61 B / 1.3 KiB / 4.0 KiB | 135 | 58 | 0 | 0 | 0 |
| birli-mwax | pure | 52 | 556.9 KiB | 91.18 | 2 B / 25.3 KiB / 25.3 KiB | 25 | 98.0 KiB | 249.06 | 60 B / 160 B / 25.3 KiB | 0 | 35 | 0 | 0 | 6 |
| hd-di-cal | cfitsio | 61 | 8.4 MiB | 6.95 | 32 B / 4.0 KiB / 8.1 MiB | 176 | 409.2 KiB | 419.99 | 30 B / 75 B / 20.0 KiB | 40 | 25 | 0 | 0 | 0 |
| hd-di-cal | pure | 29 | 8.7 MiB | 3.20 | 2 B / 64.0 KiB / 1.0 MiB | 116 | 403.6 KiB | 280.67 | 30 B / 58 B / 396.6 KiB | 1 | 16 | 0 | 0 | 1 |
| hd-sols-apply | cfitsio | 73 | 8.6 MiB | 8.08 | 2.4 KiB / 4.0 KiB / 8.1 MiB | 3175 | 12.2 MiB | 249.17 | 30 B / 4.0 KiB / 4.0 KiB | 51 | 24 | 0 | 0 | 0 |
| hd-sols-apply | pure | 30 | 8.9 MiB | 3.21 | 2 B / 64.0 KiB / 1.0 MiB | 79 | 12.2 MiB | 6.20 | 30 B / 72 B / 1.0 MiB | 0 | 14 | 0 | 0 | 0 |
| hd-srclist-convert | cfitsio | 2 | 98.0 KiB | 19.93 | 32 B / 49.0 KiB / 98.0 KiB | 17 | 31.7 KiB | 524.00 | 30 B / 210 B / 4.0 KiB | 1 | 4 | 0 | 0 | 0 |
| hd-srclist-convert | pure | 2 | 98.0 KiB | 19.93 | 32 B / 49.0 KiB / 98.0 KiB | 10 | 31.7 KiB | 308.23 | 30 B / 73 B / 30.9 KiB | 1 | 4 | 0 | 0 | 1 |
| hd-vis-convert | cfitsio | 69284 | 715.1 MiB | 92.40 | 1.3 KiB / 4.0 KiB / 32.0 KiB | 75876 | 572.0 MiB | 126.51 | 30 B / 4.0 KiB / 24.0 KiB | 41002 | 12 | 0 | 0 | 0 |
| hd-vis-convert | pure | 17710 | 581.3 MiB | 29.06 | 2 B / 64.0 KiB / 64.0 KiB | 629 | 572.0 MiB | 1.05 | 30 B / 1.0 MiB / 1.0 MiB | 0 | 7 | 0 | 0 | 0 |
| mwalib-legacy | cfitsio | 57001 | 777.7 MiB | 69.90 | 1.1 KiB / 4.0 KiB / 28.0 KiB | 108 | 453.1 KiB | 232.76 | 2 B / 7.2 KiB / 8.0 KiB | 871 | 388 | 0 | 0 | 0 |
| mwalib-legacy | pure | 1445 | 798.1 MiB | 1.73 | 2 B / 1.0 MiB / 1.0 MiB | 108 | 453.1 KiB | 232.76 | 2 B / 7.2 KiB / 8.0 KiB | 0 | 194 | 0 | 0 | 0 |
| mwalib-mwax | cfitsio | 6711 | 26.2 MiB | 244.43 | 832 B / 4.0 KiB / 4.0 KiB | 82 | 333.9 KiB | 239.82 | 2 B / 6.6 KiB / 8.0 KiB | 783 | 24 | 0 | 0 | 0 |
| mwalib-mwax | pure | 39 | 24.5 MiB | 1.52 | 2 B / 64.0 KiB / 5.8 MiB | 82 | 333.9 KiB | 239.82 | 2 B / 6.6 KiB / 8.0 KiB | 0 | 12 | 0 | 0 | 0 |

## Request sizes of reads and writes on the FITS files (number of calls)

| workload | backend | reads <512B | <4KiB | <64KiB | <1MiB | >=1MiB | writes <512B | <4KiB | <64KiB | <1MiB | >=1MiB |
|---|---|---|---|---|---|---|---|---|---|---|---|
| birli-legacy | cfitsio | 0 | 194 | 47783 | 0 | 0 | 1499 | 10104 | 151549 | 0 | 0 |
| birli-legacy | pure | 194 | 0 | 25 | 362 | 384 | 11 | 12 | 61 | 60 | 576 |
| birli-legacy-chunked | cfitsio | 0 | 218 | 47975 | 0 | 0 | 1548 | 10176 | 151549 | 0 | 0 |
| birli-legacy-chunked | pure | 218 | 0 | 49 | 362 | 384 | 36 | 12 | 85 | 60 | 576 |
| birli-mwax | cfitsio | 2 | 24 | 130 | 0 | 0 | 16 | 9 | 13 | 0 | 0 |
| birli-mwax | pure | 26 | 2 | 24 | 0 | 0 | 14 | 4 | 7 | 0 | 0 |
| hd-di-cal | cfitsio | 1 | 9 | 49 | 1 | 1 | 116 | 3 | 57 | 0 | 0 |
| hd-di-cal | pure | 10 | 0 | 4 | 7 | 8 | 115 | 0 | 0 | 1 | 0 |
| hd-sols-apply | cfitsio | 0 | 9 | 62 | 1 | 1 | 63 | 5 | 3107 | 0 | 0 |
| hd-sols-apply | pure | 9 | 0 | 4 | 9 | 8 | 63 | 0 | 3 | 1 | 12 |
| hd-srclist-convert | cfitsio | 1 | 0 | 0 | 1 | 0 | 9 | 1 | 7 | 0 | 0 |
| hd-srclist-convert | pure | 1 | 0 | 0 | 1 | 0 | 9 | 0 | 1 | 0 | 0 |
| hd-vis-convert | cfitsio | 0 | 20 | 69264 | 0 | 0 | 53 | 3 | 75820 | 0 | 0 |
| hd-vis-convert | pure | 8410 | 0 | 4 | 9296 | 0 | 54 | 0 | 3 | 1 | 571 |
| mwalib-legacy | cfitsio | 0 | 194 | 56807 | 0 | 0 | 35 | 16 | 57 | 0 | 0 |
| mwalib-legacy | pure | 194 | 0 | 1 | 482 | 768 | 35 | 16 | 57 | 0 | 0 |
| mwalib-mwax | cfitsio | 0 | 12 | 6699 | 0 | 0 | 31 | 9 | 42 | 0 | 0 |
| mwalib-mwax | pure | 12 | 0 | 1 | 22 | 4 | 31 | 9 | 42 | 0 | 0 |

## All syscalls (strace -f -c), total calls

| workload | cfitsio | pure |
|---|---|---|
| birli-legacy | 264044 | 20413 |
| birli-legacy-chunked | 267406 | 23937 |
| birli-mwax | 6330 | 5293 |
| hd-di-cal | 207978 | 213250 |
| hd-sols-apply | 19448 | 16760 |
| hd-srclist-convert | 545 | 219 |
| hd-vis-convert | 186758 | 18607 |
| mwalib-legacy | 59795 | 2271 |
| mwalib-mwax | 8125 | 311 |
