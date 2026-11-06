#!/usr/bin/env python3
"""Drop files from the page cache without root, for cold-cache runs.

posix_fadvise(POSIX_FADV_DONTNEED) evicts a file's clean cached pages; it
needs no privileges. Dirty pages are written back first (fdatasync). On
Lustre, use `lctl set_param ldlm.namespaces.*.lru_size=clear` instead if
you have the rights; DONTNEED is honoured by the client page cache too.

Usage: evict.py FILE_OR_DIR...
"""
import os
import sys


def evict(path):
    fd = os.open(path, os.O_RDONLY)
    try:
        try:
            os.fdatasync(fd)
        except OSError:
            pass
        os.posix_fadvise(fd, 0, 0, os.POSIX_FADV_DONTNEED)
    finally:
        os.close(fd)


for arg in sys.argv[1:]:
    if os.path.isdir(arg):
        for root, _, files in os.walk(arg):
            for f in files:
                evict(os.path.join(root, f))
    elif os.path.exists(arg):
        evict(arg)
