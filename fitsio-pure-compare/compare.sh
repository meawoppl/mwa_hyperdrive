#!/usr/bin/env bash
# Compare the cfitsio and fitsio-pure backends of the MWA chain
# (mwalib -> Marlu -> Birli -> mwa_hyperdrive) on real MWA data:
# output correctness, wall/CPU time, peak RSS, and the I/O syscalls each
# backend makes on the FITS files.
#
# Layout expected under $ROOT (this script's directory by default):
#   mwalib/ Marlu/ Birli/ mwa_hyperdrive/   checkouts on meawoppl/fitsio-pure-feature,
#                                           chained with [patch] in .cargo/config.toml
#   tools/mwalib-dump/                      reads every visibility through mwalib
#   scripts/                                evict.py strace_stats.py fits_compare.py summarize.py
#
# Usage:
#   ./compare.sh                     build both backends, run every workload
#   ./compare.sh --no-build          reuse $ROOT/bin/{cfitsio,pure}
#   ./compare.sh birli-legacy hd-di-cal    only these workloads
#
# Environment:
#   WORK      where workload outputs are written; put it on the filesystem
#             under test (e.g. Lustre). Default: $ROOT/work
#   RESULTS   where timings, traces and comparisons go. Default: $ROOT/results
#   REPS      timed runs per backend and cache state. Default: 3
#   PYTHON    python3 for the helper scripts (stdlib only). Default: python3
#   NUMPY_PYTHON  a python with numpy, for value diffs in fits_compare.py and
#             for generating the large uvfits. Default: $PYTHON
#   MWA_BEAM_FILE  the FEE beam HDF5 file, needed by the hd-* workloads
#   FITS_TEST_CASES  a fits-test-cases checkout, for the generated uvfits used
#             by hd-vis-convert and the hyperdrive solutions used by
#             hd-sols-apply. Those workloads are skipped without it.
#   LARGE_UVFITS_TIMESTEPS  size of the generated uvfits (300 MB each). Default: 2
#   HD_FEATURES  extra hyperdrive features. Default: hdf5-static
#   MIN_FREE_GB  stop if the filesystem holding WORK has less free space. Default: 3
#   STRACE_ONLY  set to 1 to redo only the syscall traces, keeping earlier results
set -euo pipefail

ROOT=${ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}
WORK=${WORK:-$ROOT/work}
RESULTS=${RESULTS:-$ROOT/results}
REPS=${REPS:-3}
PYTHON=${PYTHON:-python3}
NUMPY_PYTHON=${NUMPY_PYTHON:-$PYTHON}
HD_FEATURES=${HD_FEATURES:-hdf5-static}
MIN_FREE_GB=${MIN_FREE_GB:-3}
STRACE_ONLY=${STRACE_ONLY:-0}
LARGE_UVFITS_TIMESTEPS=${LARGE_UVFITS_TIMESTEPS:-2}
BIN=$ROOT/bin
SCRIPTS=$ROOT/scripts
BACKENDS=(cfitsio pure)
TRACED="read,pread64,readv,preadv,write,pwrite64,writev,pwritev,lseek,openat,open,close,fsync,fdatasync,mmap,rename,renameat,renameat2,ftruncate,fallocate"

BUILD=1
SELECTED=()
for arg in "$@"; do
    case "$arg" in
        --no-build) BUILD=0 ;;
        -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
        *) SELECTED+=("$arg") ;;
    esac
done

need() { command -v "$1" >/dev/null || { echo "compare.sh: needs $1" >&2; exit 1; }; }
need strace
[ -x /usr/bin/time ] || { echo "compare.sh: needs GNU time at /usr/bin/time" >&2; exit 1; }

# Data, all from the repositories' own test files.
PICKET=$ROOT/Birli/tests/data/1119683928_picket      # legacy, 24 x 16.9 MB gpubox
BIRLI_MWAX=$ROOT/Birli/tests/data/1297526432_mwax    # MWAX, small
MWALIB_MWAX=$ROOT/mwalib/test_files/1244973688_1_timestep
HD=$ROOT/mwa_hyperdrive/test_files/1090008640        # legacy, reduced, + mwaf + srclist
FTC=${FITS_TEST_CASES:-}

build() {
    local backend=$1 feat="" hdfeat="--features $HD_FEATURES"
    if [ "$backend" = pure ]; then
        feat="--features fitsio-pure"
        hdfeat="--features $HD_FEATURES,fitsio-pure"
    fi
    local target
    target=$(cd "$ROOT/Birli" && cargo metadata --format-version 1 --no-deps | "$PYTHON" -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
    mkdir -p "$BIN/$backend"
    echo ">> building $backend binaries"
    (cd "$ROOT/tools/mwalib-dump" && cargo build -q --release $feat)
    cp "$target/release/mwalib-dump" "$BIN/$backend/"
    (cd "$ROOT/Birli" && cargo build -q --release --bin birli $feat)
    cp "$target/release/birli" "$BIN/$backend/"
    (cd "$ROOT/mwa_hyperdrive" && cargo build -q --release --bin hyperdrive $hdfeat)
    cp "$target/release/hyperdrive" "$BIN/$backend/"
}

# Each workload sets: PROG (binary name), ARGS (array), INPUTS (array of
# files/dirs it reads), STDOUT_OUT (file name to keep stdout as, or empty).
# Outputs are written to the current directory, which is the same path for
# both backends so that command lines recorded in headers (CMDLINE) match.
define() {
    STDOUT_OUT=""
    case "$1" in
        mwalib-legacy)
            PROG=mwalib-dump; INPUTS=("$PICKET")
            ARGS=("$PICKET/1119683928.metafits" "$PICKET"/*gpubox*.fits); STDOUT_OUT=dump.txt ;;
        mwalib-mwax)
            PROG=mwalib-dump; INPUTS=("$MWALIB_MWAX")
            ARGS=("$MWALIB_MWAX/1244973688.metafits" "$MWALIB_MWAX"/*ch114*.fits); STDOUT_OUT=dump.txt ;;
        birli-legacy)
            PROG=birli; INPUTS=("$PICKET")
            ARGS=(--no-draw-progress -m "$PICKET/1119683928.metafits" -u out.uvfits -f flags_%%.mwaf "$PICKET"/*gpubox*.fits) ;;
        birli-legacy-chunked)
            PROG=birli; INPUTS=("$PICKET")
            ARGS=(--no-draw-progress --time-chunk 1 -m "$PICKET/1119683928.metafits" -u out.uvfits -f flags_%%.mwaf "$PICKET"/*gpubox*.fits) ;;
        birli-mwax)
            PROG=birli; INPUTS=("$BIRLI_MWAX")
            ARGS=(--no-draw-progress -m "$BIRLI_MWAX/1297526432.metafits" -u out.uvfits -f flags_%%%.mwaf "$BIRLI_MWAX"/*ch*.fits) ;;
        hd-di-cal)
            PROG=hyperdrive; INPUTS=("$HD")
            ARGS=(di-calibrate --no-progress-bars -d "$HD/1090008640.metafits" "$HD/1090008640_20140721201027_gpubox01_00.fits" "$HD/1090008640_01.mwaf"
                  -s "$HD/srclist_pumav3_EoR0aegean_EoR1pietro+ForA_1090008640_100.yaml" -o sols.fits) ;;
        hd-sols-apply)
            [ -n "$FTC" ] || return 1
            PROG=hyperdrive; INPUTS=("$HD" "$FTC/mwa/1090008640_hyperdrive_solutions.fits")
            ARGS=(solutions-apply --no-progress-bars -d "$HD/1090008640.metafits" "$HD/1090008640_20140721201027_gpubox01_00.fits"
                  -s "$FTC/mwa/1090008640_hyperdrive_solutions.fits" -o out.uvfits) ;;
        hd-vis-convert)
            [ -n "$FTC" ] || return 1
            local large=$WORK/data/large_${LARGE_UVFITS_TIMESTEPS}ts.uvfits
            if [ ! -f "$large" ]; then
                mkdir -p "$WORK/data"
                "$NUMPY_PYTHON" "$FTC/scripts/make_large_mwa.py" uvfits "$large" --timesteps "$LARGE_UVFITS_TIMESTEPS" >/dev/null
            fi
            PROG=hyperdrive; INPUTS=("$large")
            ARGS=(vis-convert --no-progress-bars -d "$large" -o out.uvfits) ;;
        hd-srclist-convert)
            PROG=hyperdrive; INPUTS=("$HD")
            ARGS=(srclist-convert -o fits "$HD/srclist_pumav3_EoR0aegean_EoR1pietro+ForA_1090008640_100.yaml" srclist.fits) ;;
        *) echo "unknown workload $1" >&2; return 1 ;;
    esac
}

ALL=(mwalib-legacy mwalib-mwax birli-legacy birli-legacy-chunked birli-mwax hd-di-cal hd-sols-apply hd-vis-convert hd-srclist-convert)
[ ${#SELECTED[@]} -gt 0 ] || SELECTED=("${ALL[@]}")

warm() { for i in "${INPUTS[@]}"; do find "$i" -type f -exec cat {} + >/dev/null; done; }
cold() { "$PYTHON" "$SCRIPTS/evict.py" "${INPUTS[@]}"; }

# Run the workload once in $WORK/run with the backend's binary at
# $WORK/run/$PROG. Extra leading arguments wrap the command (time, strace).
run_once() {
    local backend=$1; shift
    local free_kb
    free_kb=$(df -Pk "$WORK" | awk 'NR==2 {print $4}')
    if [ "$free_kb" -lt $((MIN_FREE_GB * 1024 * 1024)) ]; then
        echo "compare.sh: less than ${MIN_FREE_GB} GB free under $WORK; stopping" >&2
        exit 1
    fi
    rm -rf "$WORK/run"
    mkdir -p "$WORK/run"
    cp "$BIN/$backend/$PROG" "$WORK/run/$PROG"
    (
        cd "$WORK/run"
        if [ -n "$STDOUT_OUT" ]; then
            "$@" "./$PROG" "${ARGS[@]}" >"$STDOUT_OUT" 2>run.log
        else
            "$@" "./$PROG" "${ARGS[@]}" >run.log 2>&1
        fi
    )
}

outputs_of() { (cd "$WORK/run" && ls | grep -v -e "^$PROG\$" -e '^run.log$' -e '^trace.log$' || true); }

if [ "$BUILD" = 1 ]; then
    for b in "${BACKENDS[@]}"; do build "$b"; done
fi

if [ -n "${MWA_BEAM_FILE:-}" ]; then export MWA_BEAM_FILE; fi
mkdir -p "$RESULTS" "$WORK"
for wl in "${SELECTED[@]}"; do
    if ! define "$wl"; then
        echo ">> skipping $wl (missing FITS_TEST_CASES?)"
        continue
    fi
    echo ">> $wl"
    for b in "${BACKENDS[@]}"; do
        res=$RESULTS/$wl/$b
        if [ "$STRACE_ONLY" != 1 ]; then
            rm -rf "$res"
            mkdir -p "$res/outputs"

            # 1. A plain run whose outputs are kept for the correctness check.
            warm
            run_once "$b" || { echo "   $b failed; see $res/run.log"; cp "$WORK/run/run.log" "$res/" || true; continue; }
            cp "$WORK/run/run.log" "$res/"
            for f in $(outputs_of); do mv "$WORK/run/$f" "$res/outputs/"; done

            # 2. Timed runs, cold and warm page cache.
            for mode in cold warm; do
                for rep in $(seq 1 "$REPS"); do
                    "$mode"
                    run_once "$b" /usr/bin/time -v -o "$res/time_${mode}_${rep}.txt" || true
                done
            done
        fi

        # 3. Syscall summary of every call, and a full trace of the I/O calls
        #    (cold cache, though the calls a program makes don't depend on it).
        cold
        run_once "$b" strace -f -c -o "$res/strace_c.txt" || true
        cold
        run_once "$b" strace -f -yy -s 0 -e trace="$TRACED" -o "$WORK/run/trace.log" || true
        data_prefixes=()
        for i in "${INPUTS[@]}"; do data_prefixes+=(--data "$(realpath "$i")"); done
        "$PYTHON" "$SCRIPTS/strace_stats.py" "$WORK/run/trace.log" "${data_prefixes[@]}" \
            --data "$(realpath "$WORK/run")/" --json "$res/syscalls.json" >"$res/syscalls.txt"
        rm -rf "$WORK/run"
    done

    [ "$STRACE_ONLY" != 1 ] || continue
    # Correctness: every output of the cfitsio run against the fitsio-pure run.
    a=$RESULTS/$wl/cfitsio/outputs
    b=$RESULTS/$wl/pure/outputs
    {
        for f in $(ls "$a"); do
            if [ ! -e "$b/$f" ]; then echo "== $f: missing from fitsio-pure outputs"; continue; fi
            case "$f" in
                *.fits|*.uvfits|*.mwaf)
                    if out=$("$NUMPY_PYTHON" "$SCRIPTS/fits_compare.py" "$a/$f" "$b/$f"); then
                        echo "== $f: byte-identical"
                    else
                        echo "== $f: differs (see below)"
                        echo "$out" | sed 's/^/   /'
                    fi ;;
                *)
                    if cmp -s "$a/$f" "$b/$f"; then echo "== $f: byte-identical"
                    else echo "== $f: differs"; diff "$a/$f" "$b/$f" | head -20 | sed 's/^/   /'; fi ;;
            esac
        done
    } >"$RESULTS/$wl/correctness.txt"
    grep '^== ' "$RESULTS/$wl/correctness.txt" | sed 's/^/   /'  || true
    # Keep the results small: the outputs themselves are not needed any more.
    rm -rf "$RESULTS/$wl"/*/outputs
done

"$PYTHON" "$SCRIPTS/summarize.py" "$RESULTS" >"$RESULTS/summary.md"
echo ">> summary: $RESULTS/summary.md"
