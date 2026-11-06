#!/usr/bin/env python3
"""Compare two FITS files HDU by HDU and explain every difference.

Reports, per HDU: header cards only in one file, cards whose text differs
(value and comment shown separately), card-order differences, and for the
data unit whether it is byte-identical; if not, with numpy, the number of
differing values and the largest absolute and relative difference, per
column for binary tables. Exit status 0 if the files are byte-identical,
1 if they differ only as reported, 2 on a structural mismatch.

Needs only the standard library; numpy (if importable) adds value diffs.

Usage: fits_compare.py A.fits B.fits [--quiet]
"""
import hashlib
import re
import sys

try:
    import numpy as np
except ImportError:  # pragma: no cover
    np = None

BLOCK = 2880


def hdus(data):
    off = 0
    while off < len(data):
        cards = []
        while True:
            block = data[off : off + BLOCK]
            if len(block) < BLOCK:
                return
            off += BLOCK
            done = False
            for i in range(0, BLOCK, 80):
                card = block[i : i + 80].decode("ascii", "replace")
                if card.startswith("END") and card[3:].strip() == "":
                    done = True
                    break
                cards.append(card)
            if done:
                break
        kv = {}
        for c in cards:
            if c[8:10] == "= ":
                kv.setdefault(c[:8].strip(), value_of(c))
        bitpix = int(kv.get("BITPIX", 8))
        naxis = int(kv.get("NAXIS", 0))
        axes = [int(kv.get(f"NAXIS{i}", 0)) for i in range(1, naxis + 1)]
        groups = kv.get("GROUPS") == "T"
        pcount = int(kv.get("PCOUNT", 0))
        gcount = int(kv.get("GCOUNT", 1))
        if naxis == 0:
            n = 0
        else:
            prod = 1
            for a in (axes[1:] if groups else axes):
                prod *= a
            n = abs(bitpix) // 8 * gcount * (pcount + prod)
        start = off
        off += (n + BLOCK - 1) // BLOCK * BLOCK
        yield {"cards": cards, "kv": kv, "data": data[start : start + n], "axes": axes, "bitpix": bitpix,
               "groups": groups, "pcount": pcount, "gcount": gcount}


def value_of(card):
    v = card[10:]
    if v.lstrip().startswith("'"):
        m = re.match(r"\s*'((?:[^']|'')*)'", v)
        return m.group(1).rstrip() if m else v.strip()
    return v.split("/")[0].strip()


def split_card(card):
    """(keyword, value text, comment) of a card."""
    key = card[:8].strip()
    if card[8:10] != "= ":
        return key, None, card[8:].rstrip()
    v = card[10:]
    if v.lstrip().startswith("'"):
        m = re.match(r"(\s*'(?:[^']|'')*')(.*)$", v)
        val, rest = (m.group(1), m.group(2)) if m else (v, "")
    else:
        val, _, rest = v.partition("/")
        rest = "/" + rest if _ else ""
    comment = rest.strip()
    comment = comment[1:].strip() if comment.startswith("/") else comment
    return key, val.strip(), comment


TFORM = re.compile(r"^(\d*)([LXBIJKAEDCM])")
DT = {"L": "i1", "B": "u1", "I": ">i2", "J": ">i4", "K": ">i8", "E": ">f4", "D": ">f8", "C": ">c8", "M": ">c16"}


def table_dtype(kv):
    fields = []
    for i in range(1, int(kv.get("TFIELDS", 0)) + 1):
        m = TFORM.match(kv.get(f"TFORM{i}", ""))
        if not m:
            return None
        r = int(m.group(1) or 1)
        t = m.group(2)
        name = kv.get(f"TTYPE{i}", f"col{i}") or f"col{i}"
        if t == "A":
            fields.append((name, f"S{r}"))
        elif t == "X":
            fields.append((name, "u1", ((r + 7) // 8,)))
        else:
            fields.append((name, DT[t], (r,)) if r != 1 else (name, DT[t]))
    return np.dtype(fields)


def value_diff(a, b):
    a = a.astype(np.float64) if a.dtype.kind in "iuf" else a
    b = b.astype(np.float64) if b.dtype.kind in "iuf" else b
    if a.dtype.kind != "f":
        neq = (a != b)
        return int(np.count_nonzero(neq)), None, None
    both_nan = np.isnan(a) & np.isnan(b)
    neq = (a != b) & ~both_nan
    if not neq.any():
        return 0, 0.0, 0.0
    d = np.abs(a[neq] - b[neq])
    rel = d / np.maximum(np.abs(a[neq]), np.abs(b[neq]))
    return int(np.count_nonzero(neq)), float(np.nanmax(d)), float(np.nanmax(rel))


def compare(path_a, path_b, quiet=False):
    da, db = open(path_a, "rb").read(), open(path_b, "rb").read()
    out = []
    if hashlib.sha256(da).digest() == hashlib.sha256(db).digest():
        return 0, ["byte-identical"]
    ha, hb = list(hdus(da)), list(hdus(db))
    out.append(f"sizes {len(da)} vs {len(db)} bytes; {len(ha)} vs {len(hb)} HDUs")
    if len(ha) != len(hb):
        return 2, out
    status = 1
    for n, (a, b) in enumerate(zip(ha, hb)):
        ka = [split_card(c) for c in a["cards"]]
        kb = [split_card(c) for c in b["cards"]]
        da_ = {}
        for k in ka:
            da_.setdefault(k[0], []).append(k)
        db_ = {}
        for k in kb:
            db_.setdefault(k[0], []).append(k)
        notes = []
        for key in dict.fromkeys([k[0] for k in ka] + [k[0] for k in kb]):
            la, lb = da_.get(key, []), db_.get(key, [])
            if key in ("COMMENT", "HISTORY", ""):
                if [x[2] for x in la] != [x[2] for x in lb]:
                    notes.append(f"  {key or 'blank'} cards differ: {len(la)} vs {len(lb)}")
                continue
            if not la or not lb:
                notes.append(f"  {key}: only in {'A' if la else 'B'}: {(la or lb)[0][1:]}")
                continue
            for x, y in zip(la, lb):
                if x[1] != y[1]:
                    notes.append(f"  {key} value: {x[1]!r} vs {y[1]!r}")
                if x[2] != y[2]:
                    notes.append(f"  {key} comment: {x[2]!r} vs {y[2]!r}")
        if [k[0] for k in ka] != [k[0] for k in kb] and sorted(k[0] for k in ka) == sorted(k[0] for k in kb):
            notes.append("  card order differs")
        if a["cards"] == b["cards"]:
            pass
        elif not notes:
            notes.append("  header text differs only in layout (padding/alignment)")
        if a["data"] == b["data"]:
            dnote = "data identical"
        elif np is None or len(a["data"]) != len(b["data"]):
            first = next((i for i, (x, y) in enumerate(zip(a["data"], b["data"])) if x != y), None)
            dnote = f"data differ ({len(a['data'])} vs {len(b['data'])} bytes, first at {first})"
        elif a["kv"].get("XTENSION") == "BINTABLE":
            dt = table_dtype(a["kv"])
            rows = int(a["kv"].get("NAXIS2", 0))
            ta = np.frombuffer(a["data"], dtype=dt, count=rows)
            tb = np.frombuffer(b["data"], dtype=dt, count=rows)
            parts = []
            for name in dt.names:
                cnt, mx, rel = value_diff(ta[name], tb[name])
                if cnt:
                    parts.append(f"{name}: {cnt} differ (max abs {mx}, max rel {rel})")
            dnote = "data differ: " + "; ".join(parts) if parts else "data differ in padding bytes only"
        else:
            dtype = {8: "u1", 16: ">i2", 32: ">i4", 64: ">i8", -32: ">f4", -64: ">f8"}[a["bitpix"]]
            cnt, mx, rel = value_diff(np.frombuffer(a["data"], dtype), np.frombuffer(b["data"], dtype))
            dnote = f"data differ: {cnt} values (max abs {mx}, max rel {rel})"
        out.append(f"HDU {n} ({a['kv'].get('EXTNAME', 'primary')}): {dnote}")
        out.extend(notes)
    return status, out


if __name__ == "__main__":
    a, b = sys.argv[1], sys.argv[2]
    code, lines = compare(a, b, "--quiet" in sys.argv)
    print("\n".join(lines))
    sys.exit(code)
