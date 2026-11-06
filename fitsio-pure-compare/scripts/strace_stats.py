#!/usr/bin/env python3
"""Summarise the I/O syscalls in an `strace -f -yy -s 0 -o LOG` trace.

Every call is attributed to the file its descriptor names (strace -yy
prints the path next to each fd). Calls on files under one of the --data
prefixes (the workload's inputs and outputs) are reported separately from
everything else (shared libraries, /proc, the terminal, ...), since the
Lustre question is about the FITS files.

For read/pread64/write/pwrite64 it reports the number of calls, the bytes
moved (sum of return values), and the smallest, median and largest request
sizes (the count argument), plus calls per MB moved.

Usage: strace_stats.py LOG --data PREFIX [--data PREFIX...] [--json OUT]
"""
import argparse
import json
import re
import statistics
import sys
from collections import defaultdict

LINE = re.compile(r"^(\d+)\s+(.*)$")
CALL = re.compile(r"^(\w+)\((.*)\)\s+=\s+(-?\d+|0x[0-9a-f]+|\?)(.*)$")
FD = re.compile(r"^(-?\d+)<([^>]*)>")
STR = re.compile(r'"((?:[^"\\]|\\.)*)"')
SIZED = {"read", "pread64", "write", "pwrite64"}
VECTOR = {"readv", "preadv", "preadv2", "writev", "pwritev", "pwritev2"}
READS = {"read", "pread64", "readv", "preadv", "preadv2"}
WRITES = {"write", "pwrite64", "writev", "pwritev", "pwritev2"}


def split_args(args):
    """Split a syscall's argument text on top-level commas."""
    out, depth, cur, quote = [], 0, [], False
    i = 0
    while i < len(args):
        ch = args[i]
        if quote:
            cur.append(ch)
            if ch == "\\":
                cur.append(args[i + 1])
                i += 1
            elif ch == '"':
                quote = False
        elif ch == '"':
            quote = True
            cur.append(ch)
        elif ch in "[{(":
            depth += 1
            cur.append(ch)
        elif ch in "]})":
            depth -= 1
            cur.append(ch)
        elif ch == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
        else:
            cur.append(ch)
        i += 1
    if cur:
        out.append("".join(cur).strip())
    return out


def parse(path):
    pending = {}
    with open(path, errors="replace") as f:
        for raw in f:
            m = LINE.match(raw.rstrip("\n"))
            if not m:
                continue
            pid, rest = m.groups()
            if rest.endswith("<unfinished ...>"):
                pending[pid] = rest[: -len("<unfinished ...>")].rstrip()
                continue
            r = re.match(r"^<\.\.\. (\w+) resumed>(.*)$", rest)
            if r:
                head = pending.pop(pid, None)
                if head is None:
                    continue
                rest = head + r.group(2)
            c = CALL.match(rest)
            if c:
                yield c.group(1), split_args(c.group(2)), c.group(3)


def file_of(name, args, ret):
    if name in ("openat", "open", "creat"):
        s = [a for a in args if a.startswith('"')]
        return STR.match(s[0]).group(1) if s else None
    if name in ("rename", "renameat", "renameat2"):
        s = [STR.match(a).group(1) for a in args if a.startswith('"')]
        return s[-1] if s else None
    if name == "mmap":
        fd = FD.match(args[4]) if len(args) > 4 else None
        return fd.group(2) if fd else None
    fd = FD.match(args[0]) if args else None
    return fd.group(2) if fd else None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("log")
    ap.add_argument("--data", action="append", default=[])
    ap.add_argument("--json")
    opts = ap.parse_args()

    stats = {k: defaultdict(lambda: {"calls": 0, "bytes": 0, "sizes": []}) for k in ("data", "other")}
    files = {"data": set(), "other": set()}
    for name, args, ret in parse(opts.log):
        path = file_of(name, args, ret)
        # A relative path is relative to the workload's run directory, which
        # holds only its outputs.
        is_data = path and (not path.startswith("/") or any(path.startswith(p) for p in opts.data))
        kind = "data" if is_data else "other"
        if path:
            files[kind].add(path)
        s = stats[kind][name]
        s["calls"] += 1
        try:
            n = int(ret, 0)
        except ValueError:
            n = -1
        if name in SIZED or name in VECTOR:
            if n > 0:
                s["bytes"] += n
            if name in SIZED and len(args) >= 3:
                try:
                    s["sizes"].append(int(args[2]))
                except ValueError:
                    pass

    out = {}
    for kind in ("data", "other"):
        k = {}
        for name, s in sorted(stats[kind].items()):
            entry = {"calls": s["calls"]}
            if name in SIZED or name in VECTOR:
                entry["bytes"] = s["bytes"]
                if s["sizes"]:
                    entry["min_req"] = min(s["sizes"])
                    entry["median_req"] = int(statistics.median(s["sizes"]))
                    entry["max_req"] = max(s["sizes"])
                entry["mean_bytes"] = round(s["bytes"] / s["calls"]) if s["calls"] else 0
            k[name] = entry
        hist = {}
        for label, names in (("read", ("read", "pread64")), ("write", ("write", "pwrite64"))):
            sizes = [x for n in names for x in stats[kind][n]["sizes"]] if any(n in stats[kind] for n in names) else []
            buckets = [("<512B", 512), ("<4KiB", 4096), ("<64KiB", 65536), ("<1MiB", 1 << 20), (">=1MiB", None)]
            counts = {b: 0 for b, _ in buckets}
            for x in sizes:
                for b, lim in buckets:
                    if lim is None or x < lim:
                        counts[b] += 1
                        break
            hist[label] = counts
        k["_hist"] = hist
        reads = sum(v["calls"] for n, v in k.items() if n in READS)
        writes = sum(v["calls"] for n, v in k.items() if n in WRITES)
        rbytes = sum(v.get("bytes", 0) for n, v in k.items() if n in READS)
        wbytes = sum(v.get("bytes", 0) for n, v in k.items() if n in WRITES)
        k["_summary"] = {
            "files": len(files[kind]),
            "read_calls": reads,
            "read_bytes": rbytes,
            "reads_per_MB": round(reads / (rbytes / 1e6), 2) if rbytes else None,
            "mean_read": round(rbytes / reads) if reads else None,
            "write_calls": writes,
            "write_bytes": wbytes,
            "writes_per_MB": round(writes / (wbytes / 1e6), 2) if wbytes else None,
            "mean_write": round(wbytes / writes) if writes else None,
            "total_calls": sum(v["calls"] for n, v in k.items() if not n.startswith("_")),
        }
        out[kind] = k

    if opts.json:
        with open(opts.json, "w") as f:
            json.dump(out, f, indent=1)
    for kind in ("data", "other"):
        print(f"[{kind} files]")
        for name, v in out[kind].items():
            print(f"  {name:12} {v}")


if __name__ == "__main__":
    sys.exit(main())
