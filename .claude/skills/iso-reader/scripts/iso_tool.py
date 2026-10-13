#!/usr/bin/env python3
"""Read ISO 9660 disk images without mounting. Cross-platform (Windows, Linux, macOS).

Requires: pip install pycdlib  (pure Python, no admin rights, no mounting)

Usage:
    python iso_tool.py info <image.iso>
    python iso_tool.py list <image.iso> [--dir /PATH/IN/ISO]
    python iso_tool.py extract <image.iso> <iso-path> <output-file>
    python iso_tool.py extract-all <image.iso> <output-dir>
"""
import argparse
import os
import sys
from pathlib import Path

try:
    import pycdlib
except ImportError:
    sys.exit("pycdlib is not installed. Run: pip install pycdlib")


def open_iso(path):
    iso = pycdlib.PyCdlib()
    iso.open(str(path))
    return iso


def cmd_info(args):
    iso = open_iso(args.image)
    try:
        pvd = iso.pvd
        block_size = pvd.logical_block_size()
        print("volume_id: %s" % pvd.volume_identifier.decode().strip())
        print("system_id: %s" % pvd.system_identifier.decode().strip())
        print("volume_size_blocks: %d" % pvd.space_size)
        print("block_size: %d" % block_size)
        print("total_bytes: %d" % (pvd.space_size * block_size))
    finally:
        iso.close()


def _walk(iso, iso_path, prefix=""):
    for child in iso.list_children(iso_path=iso_path):
        if child.is_dot() or child.is_dotdot():
            continue
        name = child.file_identifier().decode().strip()
        full = prefix + "/" + name if prefix else "/" + name
        is_dir = child.is_dir()
        size = 0 if is_dir else child.get_data_length()
        yield full, is_dir, size
        if is_dir:
            yield from _walk(iso, iso_path=full, prefix=full)


def cmd_list(args):
    iso = open_iso(args.image)
    try:
        start = args.dir or "/"
        for path, is_dir, size in _walk(iso, iso_path=start, prefix="" if start == "/" else start):
            kind = "DIR " if is_dir else "FILE"
            print("%s %10d %s" % (kind, size, path))
    finally:
        iso.close()


def cmd_extract(args):
    iso = open_iso(args.image)
    try:
        out = Path(args.output)
        out.parent.mkdir(parents=True, exist_ok=True)
        iso.get_file_from_iso(str(out), iso_path=args.iso_path)
        print("extracted %s -> %s (%d bytes)" % (args.iso_path, out, out.stat().st_size))
    finally:
        iso.close()


def cmd_extract_all(args):
    iso = open_iso(args.image)
    try:
        dest = Path(args.output_dir)
        count = 0
        for path, is_dir, _size in _walk(iso, iso_path="/"):
            # Strip the ";1" version suffix pycdlib appends to file names.
            rel = path.lstrip("/").split(";")[0]
            target = dest / Path(*rel.split("/"))
            if is_dir:
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                iso.get_file_from_iso(str(target), iso_path=path)
                count += 1
        print("extracted %d files to %s" % (count, dest))
    finally:
        iso.close()


def main():
    ap = argparse.ArgumentParser(description="Read ISO 9660 images without mounting.")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("info", help="Show volume metadata.")
    p.add_argument("image")
    p.set_defaults(func=cmd_info)

    p = sub.add_parser("list", help="List files and directories in the image.")
    p.add_argument("image")
    p.add_argument("--dir", default="/", help="Directory inside the ISO to list from.")
    p.set_defaults(func=cmd_list)

    p = sub.add_parser("extract", help="Extract one file from the image.")
    p.add_argument("image")
    p.add_argument("iso_path", help="Path inside the ISO, e.g. /README.TXT or /DOCS/CONFIG.INI;1")
    p.add_argument("output", help="Where to write the file on disk.")
    p.set_defaults(func=cmd_extract)

    p = sub.add_parser("extract-all", help="Extract every file to a directory.")
    p.add_argument("image")
    p.add_argument("output_dir")
    p.set_defaults(func=cmd_extract_all)

    args = ap.parse_args()
    if not os.path.isfile(args.image):
        sys.exit("not found: %s" % args.image)
    args.func(args)


if __name__ == "__main__":
    main()
