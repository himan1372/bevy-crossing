#!/usr/bin/env python3
"""gc_iso_reader.py - read Nintendo GameCube (GCM/ISO) disc images.

Pure Python standard library only: runs on Windows, macOS and Linux with no
build step and no third-party packages.

Commands:
    info <image>                  show header: game id/name, maker, disc#, version, DOL/FST offsets
    ls <image> [dir]              list files (optionally under one directory)
    extract <image> <path> -o <out>   extract one file by its disc path
    dol <image> -o <out>          extract main.dol

The layout mirrors bevy-crossing's Rust `disc` crate (disc/src/image.rs), so
both readers agree on offsets:
    header magic 0xC2339F3D at 0x1C, game id at 0x00, DOL offset at 0x420,
    FST offset/size at 0x424/0x428, 12-byte FST entries, null-terminated
    name table after the entries.

Never commit a real game image to a repo. Test images are synthetic.
"""

import argparse
import os
import struct
import sys

GC_MAGIC = 0xC2339F3D
MAX_FST_SIZE = 128 * 1024 * 1024
MAX_DIR_DEPTH = 32


class IsoError(Exception):
    pass


def be32(b):
    return struct.unpack(">I", bytes(b[:4]))[0]


class GcmImage:
    def __init__(self, path):
        self.path = path
        self.f = open(path, "rb")
        try:
            self._read_header()
            self._read_fst()
        except Exception:
            self.f.close()
            raise

    def close(self):
        self.f.close()

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.close()

    def _read_at(self, offset, size):
        self.f.seek(offset)
        data = self.f.read(size)
        if len(data) != size:
            raise IsoError("truncated image: read at 0x%X past end of file" % offset)
        return data

    def _read_header(self):
        header = self._read_at(0, 0x20)
        if be32(header[0x1C:]) != GC_MAGIC:
            raise IsoError("not a GameCube image (bad magic at 0x1C)")
        self.game_id = header[0:6].decode("ascii", "replace")
        self.maker = header[6:8].decode("ascii", "replace")
        self.disc_number = header[8]
        self.version = header[9]
        self.audio_streaming = bool(header[10])
        name_raw = self._read_at(0x20, 0x3E0)
        self.game_name = name_raw.split(b"\x00", 1)[0].decode("ascii", "replace")
        boot = self._read_at(0x420, 12)
        self.dol_offset = be32(boot[0:])
        self.fst_offset = be32(boot[4:])
        self.fst_size = be32(boot[8:])

    def _read_fst(self):
        if not 12 <= self.fst_size <= MAX_FST_SIZE:
            raise IsoError("bad FST size: %d" % self.fst_size)
        fst = self._read_at(self.fst_offset, self.fst_size)
        count = be32(fst[8:])
        if count == 0 or count * 12 > len(fst):
            raise IsoError("bad FST entry count: %d" % count)
        strings = fst[count * 12 :]

        def name_at(off):
            end = strings.find(b"\x00", off)
            if end < 0:
                raise IsoError("FST name runs past string table")
            return strings[off:end].decode("ascii", "replace")

        self.files = []  # (path, disc_offset, size)
        dirs = [(count, "")]
        for index in range(1, count):
            while dirs and index >= dirs[-1][0]:
                dirs.pop()
            if len(dirs) > MAX_DIR_DEPTH:
                raise IsoError("FST directory nesting too deep")
            node = fst[index * 12 : index * 12 + 12]
            name_off = (node[1] << 16) | (node[2] << 8) | node[3]
            name = name_at(name_off)
            field_a = be32(node[4:])
            field_b = be32(node[8:])
            if node[0] == 1:
                if field_b <= index or field_b > count:
                    raise IsoError("bad FST directory entry %d" % index)
                dirs.append((field_b, name))
            else:
                parts = [d for _, d in dirs[1:]] + [name]
                self.files.append(("/".join(parts), field_a, field_b))

    def find(self, path):
        want = path.lstrip("/").lower()
        for p, off, size in self.files:
            if p.lower() == want:
                return (p, off, size)
        return None

    def read_file(self, path):
        found = self.find(path)
        if found is None:
            raise IsoError("file not found on disc: %s" % path)
        _, off, size = found
        return self._read_at(off, size)

    def read_dol(self):
        # DOL size = end of the last of its 18 sections (7 text + 11 data).
        hdr = self._read_at(self.dol_offset, 0x100)
        end = 0
        for i in range(18):
            off = be32(hdr[i * 4 :])
            size = be32(hdr[0x90 + i * 4 :])
            end = max(end, off + size)
        return self._read_at(self.dol_offset, end)


def cmd_info(args):
    with GcmImage(args.image) as img:
        print("file:           %s" % args.image)
        print("game id:        %s" % img.game_id)
        print("game name:      %s" % img.game_name)
        print("maker:          %s" % img.maker)
        print("disc number:    %d" % img.disc_number)
        print("version:        %d" % img.version)
        print("audio stream:   %s" % img.audio_streaming)
        print("dol offset:     0x%X" % img.dol_offset)
        print("fst offset:     0x%X  size: 0x%X" % (img.fst_offset, img.fst_size))
        print("files:          %d" % len(img.files))


def cmd_ls(args):
    with GcmImage(args.image) as img:
        prefix = (args.dir or "").strip("/").lower()
        shown = 0
        for path, _off, size in img.files:
            if prefix and not (
                path.lower() == prefix or path.lower().startswith(prefix + "/")
            ):
                continue
            print("%10d  %s" % (size, path))
            shown += 1
        if args.dir and shown == 0:
            raise IsoError("no such directory on disc: %s" % args.dir)


def cmd_extract(args):
    with GcmImage(args.image) as img:
        data = img.read_file(args.path)
    outdir = os.path.dirname(os.path.abspath(args.output))
    os.makedirs(outdir, exist_ok=True)
    with open(args.output, "wb") as f:
        f.write(data)
    print("wrote %d bytes to %s" % (len(data), args.output))


def cmd_dol(args):
    with GcmImage(args.image) as img:
        data = img.read_dol()
    outdir = os.path.dirname(os.path.abspath(args.output))
    os.makedirs(outdir, exist_ok=True)
    with open(args.output, "wb") as f:
        f.write(data)
    print("wrote %d bytes to %s" % (len(data), args.output))


def main(argv=None):
    ap = argparse.ArgumentParser(description="Read GameCube (GCM/ISO) disc images")
    sub = ap.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("info", help="show disc header")
    p.add_argument("image")
    p.set_defaults(func=cmd_info)

    p = sub.add_parser("ls", help="list files on the disc")
    p.add_argument("image")
    p.add_argument("dir", nargs="?", default="")
    p.set_defaults(func=cmd_ls)

    p = sub.add_parser("extract", help="extract one file")
    p.add_argument("image")
    p.add_argument("path", help="disc path, e.g. opening.bnr or dir/file.bin")
    p.add_argument("-o", "--output", required=True)
    p.set_defaults(func=cmd_extract)

    p = sub.add_parser("dol", help="extract main.dol")
    p.add_argument("image")
    p.add_argument("-o", "--output", required=True)
    p.set_defaults(func=cmd_dol)

    args = ap.parse_args(argv)
    try:
        args.func(args)
    except IsoError as e:
        print("error: %s" % e, file=sys.stderr)
        return 1
    except FileNotFoundError:
        print("error: file not found: %s" % args.image, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
