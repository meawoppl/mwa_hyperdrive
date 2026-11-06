#!/usr/bin/env python3
"""Turn compare.sh's results directory into markdown tables.

Usage: summarize.py RESULTS_DIR > summary.md
"""
import json
import os
import re
import statistics
import sys

BACKENDS = ("cfitsio", "pure")


def parse_time(path):
    """Wall seconds, user, sys and peak RSS (MB) from GNU time -v output."""
    text = open(path).read()

    def grab(label):
        m = re.search(re.escape(label) + r":\s*(\S+)", text)
        return m.group(1) if m else None

    wall = grab("Elapsed (wall clock) time (h:mm:ss or m:ss)")
    secs = 0.0
    for part in wall.split(":"):
        secs = secs * 60 + float(part)
    status = int(grab("Exit status") or 0)
    return {
        "wall": secs,
        "user": float(grab("User time (seconds)")),
        "sys": float(grab("System time (seconds)")),
        "rss_mb": int(grab("Maximum resident set size (kbytes)")) / 1024,
        "status": status,
    }


def median_runs(dirpath, mode):
    runs = [parse_time(os.path.join(dirpath, f)) for f in sorted(os.listdir(dirpath))
            if f.startswith(f"time_{mode}_")]
    if not runs:
        return None
    out = {k: statistics.median(r[k] for r in runs) for k in ("wall", "user", "sys", "rss_mb")}
    out["n"] = len(runs)
    out["failed"] = sum(1 for r in runs if r["status"] != 0)
    return out


def fmt(v, digits=2):
    if v is None:
        return "-"
    if isinstance(v, float):
        return f"{v:.{digits}f}"
    return str(v)


def size(n):
    if n is None:
        return "-"
    for unit in ("B", "KiB", "MiB", "GiB"):
        if abs(n) < 1024 or unit == "GiB":
            return f"{n:.0f} {unit}" if unit == "B" else f"{n:.1f} {unit}"
        n /= 1024


def main():
    root = sys.argv[1]
    workloads = sorted(d for d in os.listdir(root) if os.path.isdir(os.path.join(root, d)))
    print("## Correctness\n")
    print("| workload | output | result |")
    print("|---|---|---|")
    for wl in workloads:
        cmp_path = os.path.join(root, wl, "correctness.txt")
        if not os.path.exists(cmp_path):
            continue
        for line in open(cmp_path):
            if line.startswith("== "):
                name, _, verdict = line[3:].rstrip().partition(": ")
                print(f"| {wl} | {name} | {verdict} |")
    print()

    for mode in ("cold", "warm"):
        print(f"## Time and memory ({mode} page cache, median of runs)\n")
        print("| workload | backend | wall s | user s | sys s | peak RSS MB |")
        print("|---|---|---|---|---|---|")
        for wl in workloads:
            for b in BACKENDS:
                d = os.path.join(root, wl, b)
                if not os.path.isdir(d):
                    continue
                m = median_runs(d, mode)
                if m:
                    fail = f" ({m['failed']} failed)" if m["failed"] else ""
                    print(f"| {wl} | {b} | {fmt(m['wall'])}{fail} | {fmt(m['user'])} | {fmt(m['sys'])} | {fmt(m['rss_mb'], 0)} |")
        print()

    print("## I/O syscalls on the workload's FITS files\n")
    print("| workload | backend | reads | read bytes | reads/MB | read size min/median/max | "
          "writes | write bytes | writes/MB | write size min/median/max | lseek | openat | mmap | fsync+fdatasync | rename |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for wl in workloads:
        for b in BACKENDS:
            p = os.path.join(root, wl, b, "syscalls.json")
            if not os.path.exists(p):
                continue
            d = json.load(open(p))["data"]
            s = d["_summary"]

            def sizes(names):
                mins = [d[n]["min_req"] for n in names if n in d and "min_req" in d[n]]
                meds = [d[n]["median_req"] for n in names if n in d and "median_req" in d[n]]
                maxs = [d[n]["max_req"] for n in names if n in d and "max_req" in d[n]]
                if not mins:
                    return "-"
                return f"{size(min(mins))} / {size(max(meds))} / {size(max(maxs))}"

            def calls(*names):
                return sum(d[n]["calls"] for n in names if n in d)

            print(f"| {wl} | {b} | {s['read_calls']} | {size(s['read_bytes'])} | {fmt(s['reads_per_MB'])} | "
                  f"{sizes(['read', 'pread64'])} | {s['write_calls']} | {size(s['write_bytes'])} | "
                  f"{fmt(s['writes_per_MB'])} | {sizes(['write', 'pwrite64'])} | {calls('lseek')} | "
                  f"{calls('openat', 'open')} | {calls('mmap')} | {calls('fsync', 'fdatasync')} | "
                  f"{calls('rename', 'renameat', 'renameat2')} |")
    print()

    print("## Request sizes of reads and writes on the FITS files (number of calls)\n")
    print("| workload | backend | reads <512B | <4KiB | <64KiB | <1MiB | >=1MiB | writes <512B | <4KiB | <64KiB | <1MiB | >=1MiB |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for wl in workloads:
        for b in BACKENDS:
            p = os.path.join(root, wl, b, "syscalls.json")
            if not os.path.exists(p):
                continue
            h = json.load(open(p))["data"].get("_hist")
            if not h:
                continue
            cells = [str(v) for v in h["read"].values()] + [str(v) for v in h["write"].values()]
            print(f"| {wl} | {b} | " + " | ".join(cells) + " |")
    print()

    print("## All syscalls (strace -f -c), total calls\n")
    print("| workload | cfitsio | pure |")
    print("|---|---|---|")
    for wl in workloads:
        row = []
        for b in BACKENDS:
            p = os.path.join(root, wl, b, "strace_c.txt")
            total = "-"
            if os.path.exists(p):
                for line in open(p):
                    if line.strip().endswith("total"):
                        parts = line.split()
                        total = parts[3]
            row.append(total)
        print(f"| {wl} | {row[0]} | {row[1]} |")


if __name__ == "__main__":
    main()
