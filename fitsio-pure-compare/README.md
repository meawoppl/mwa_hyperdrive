# cfitsio vs fitsio-pure: MWA chain comparison

This compares the two FITS backends of the MWA chain (mwalib → Marlu → Birli →
mwa_hyperdrive) on real MWA data, using the opt-in `fitsio-pure` feature from these branches:

| crate | branch |
|---|---|
| mwalib | [meawoppl/mwalib `meawoppl/fitsio-pure-feature`](https://github.com/meawoppl/mwalib/tree/meawoppl/fitsio-pure-feature) |
| Marlu | [meawoppl/Marlu `meawoppl/fitsio-pure-feature`](https://github.com/meawoppl/Marlu/tree/meawoppl/fitsio-pure-feature) |
| Birli | [meawoppl/Birli `meawoppl/fitsio-pure-feature`](https://github.com/meawoppl/Birli/tree/meawoppl/fitsio-pure-feature) |
| mwa_hyperdrive | [meawoppl/mwa_hyperdrive `meawoppl/fitsio-pure-feature`](https://github.com/meawoppl/mwa_hyperdrive/tree/meawoppl/fitsio-pure-feature) |

cfitsio stays the default in every crate. `--features fitsio-pure` swaps the backend and is
forwarded down the chain.

For each of 9 workloads, the script builds both backends and measures:

- **output correctness:** every output file is compared byte for byte, and any difference is
  explained;
- **wall and CPU time, and peak RSS:** using `/usr/bin/time -v`, cold and warm;
- **I/O syscalls on the FITS files:** count, size and bytes per call, using `strace`.

The workloads are mwalib reading legacy and MWAX visibilities; Birli writing uvfits and mwaf from
legacy (whole and chunked) and MWAX data; and hyperdrive di-calibrate, solutions-apply, vis-convert
and srclist-convert.

## Layout

Put the four checkouts next to this directory's contents, so that `.cargo/config.toml` chains
them with `[patch]`:

```
fitsio-pure-compare/
  .cargo/config.toml      [patch] mwalib/Marlu/Birli -> ./mwalib ./Marlu ./Birli; target-dir ./target
  compare.sh
  scripts/                evict.py strace_stats.py fits_compare.py summarize.py
  tools/mwalib-dump/      reads every visibility through mwalib and hashes it
  mwalib/  Marlu/  Birli/  mwa_hyperdrive/     the four branches above
```

```sh
for r in mwalib Marlu Birli mwa_hyperdrive; do
  git clone -b meawoppl/fitsio-pure-feature https://github.com/meawoppl/$r
done
```

## Run

```sh
MWA_BEAM_FILE=/path/mwa_full_embedded_element_pattern.h5 \
FITS_TEST_CASES=/path/fits-test-cases \
NUMPY_PYTHON=/path/to/python-with-numpy \
WORK=/lustre/scratch/fitsio-pure-compare REPS=3 ./compare.sh
```

- `WORK` holds the workload outputs. Put it on the filesystem you want to measure, e.g. Lustre.
- `FITS_TEST_CASES` is a checkout of
  [OrbitalCommons/fits-test-cases](https://github.com/OrbitalCommons/fits-test-cases). The test data
  comes from it and from the crates' own test files.

Options:

- `--no-build` reuses the binaries already built.
- Name workloads (e.g. `./compare.sh birli-legacy hd-di-cal`) to run only those.
- `STRACE_ONLY=1` re-runs just the syscall traces.
- `MIN_FREE_GB` (default 3) stops the run when disk space runs low.

On Lustre, `lctl set_param ldlm.namespaces.*.lru_size=clear` gives a truer cold cache than the
`posix_fadvise` eviction the script does by default.

Results go to `results/summary.md`, plus per-workload `correctness.txt` and `syscalls.json`.

## Reference results

`reference-results/summary-x86_64-nvme.md` is a full run on an x86_64 machine with local NVMe and
fitsio-pure 0.25.0. Highlights:

- **Correctness:** 80/80 outputs are byte-identical between the backends.
- **Speed:** wall times are within noise, except mwalib MWAX, where fitsio-pure is slower (0.53 →
  0.67 s).
- **Writes:** ~1 MiB requests, about 1 per MB, against 127–255 per MB with cfitsio.
- **Reads:** pread-based, with no lseek or fsync calls; for example Birli makes 965 reads against
  48k.
