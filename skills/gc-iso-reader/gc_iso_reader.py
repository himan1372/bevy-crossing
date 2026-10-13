#!/usr/bin/env python3
"""gc_iso_reader.py - read Nintendo GameCube (GCM/ISO) disc images.

Pure Python standard library only: runs on Windows, macOS and Linux with no
build step and no third-party packages.

Commands:
    info <image>                  show header: game id/name, maker, disc#, version, DOL/FST offsets
    ls <image> [dir]              list files (optionally under one directory)
    extract <image> <path> -o <out>   extract one file by its disc path
    dol <image> -o <out>          extract main.dol
    sym <image> <pattern> [-x N]  find symbols (disc/data/*.sym) and hex-dump them
    hex <image> rel|dol <offset> [-n N]  hex-dump the REL or DOL with pointer names
    arc <image> <file.arc> [member] [-x N]  list a RARC archive, or dump one member

<image> may be "rom": $BEVY_CROSSING_ISO, else the only .iso/.gcm in ./rom.
sym/hex/arc only print to stdout. The decompressed foresta.rel is cached in
the temp folder (never the repo) so later calls are fast.

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
import tempfile

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
        # Game code is bytes 0-3, maker code 4-5 (the id's last two).
        self.maker = header[4:6].decode("ascii", "replace")
        self.disc_number = header[6]
        self.version = header[7]
        self.audio_streaming = bool(header[8])
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


# ---- Yaz0, RARC, DOL/REL and symbol lookups (mirror disc/src/*.rs) ----

R_PPC_ADDR32 = 1
R_DOLPHIN_NOP = 201
R_DOLPHIN_SECTION = 202
R_DOLPHIN_END = 203
SYM_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "disc", "data")


def be16(b):
    return struct.unpack(">H", bytes(b[:2]))[0]


def yaz0_decode(src):
    """Yaz0 (.szs) decompression, as disc/src/yaz0.rs."""
    if len(src) < 16 or src[:4] != b"Yaz0":
        raise IsoError("not Yaz0 data")
    out_len = be32(src[4:])
    dst = bytearray()
    sp = 16
    try:
        while len(dst) < out_len:
            flags = src[sp]
            sp += 1
            for bit in range(7, -1, -1):
                if len(dst) >= out_len:
                    break
                if flags & (1 << bit):
                    dst.append(src[sp])
                    sp += 1
                    continue
                first, second = src[sp], src[sp + 1]
                sp += 2
                distance = ((first & 0x0F) << 8 | second) + 1
                if first >> 4 == 0:
                    length = src[sp] + 0x12
                    sp += 1
                else:
                    length = (first >> 4) + 2
                start = len(dst) - distance
                if start < 0:
                    raise IsoError("bad Yaz0 back-reference")
                for i in range(min(length, out_len - len(dst))):
                    dst.append(dst[start + i])
    except IndexError:
        raise IsoError("truncated Yaz0 data")
    return bytes(dst)


def rarc_files(data):
    """RARC member list [(name, offset, size)], as disc/src/rarc.rs."""
    if data[:4] == b"Yaz0":
        data = yaz0_decode(data)
    if data[:4] != b"RARC":
        raise IsoError("not a RARC archive")
    data_start = 0x20 + be32(data[0x0C:])
    count = be32(data[0x28:])
    entries = 0x20 + be32(data[0x2C:])
    strings = 0x20 + be32(data[0x34:])
    files = []
    for i in range(count):
        e = entries + i * 0x14
        if be16(data[e + 4 :]) & 0x0200:
            continue
        name_at = strings + be16(data[e + 6 :])
        name = data[name_at : data.index(b"\x00", name_at)].decode("ascii", "replace")
        off, size = data_start + be32(data[e + 8 :]), be32(data[e + 12 :])
        if off + size > len(data):
            raise IsoError("RARC member %s runs past the archive" % name)
        files.append((name, off, size))
    return data, files


class Rom:
    """main.dol + decompressed foresta.rel with REL data pointers
    (R_PPC_ADDR32 only), as disc/src/rom.rs."""

    def __init__(self, img):
        self.dol = img.read_dol()
        self.rel = self._cached_rel(img)
        self.dol_sections = [
            (be32(self.dol[i * 4 :]), be32(self.dol[0x48 + i * 4 :]), be32(self.dol[0x90 + i * 4 :]))
            for i in range(18)
        ]
        self.dol_sections = [s for s in self.dol_sections if s[2]]
        rel = self.rel
        self.rel_id = be32(rel)
        count, table = be32(rel[0x0C:]), be32(rel[0x10:])
        self.rel_sections = [be32(rel[table + i * 8 :]) & ~1 for i in range(count)]
        self.pointers = self._relocations()

    @staticmethod
    def _cached_rel(img):
        st = os.stat(img.path)
        cache = os.path.join(
            tempfile.gettempdir(), "gc_iso_reader_%s_%d_%d.rel" % (img.game_id, st.st_size, int(st.st_mtime))
        )
        if os.path.exists(cache):
            with open(cache, "rb") as f:
                return f.read()
        rel = yaz0_decode(img.read_file("foresta.rel.szs"))
        with open(cache, "wb") as f:
            f.write(rel)
        return rel

    def dol_offset(self, addr):
        for off, base, size in self.dol_sections:
            if base <= addr < base + size:
                return off + addr - base
        return None

    def _relocations(self):
        rel = self.rel
        imp, imp_size = struct.unpack_from(">II", rel, 0x28)
        out = {}
        for k in range(imp, imp + imp_size, 8):
            module, at = struct.unpack_from(">II", rel, k)
            base = None
            while True:
                # unpack_from: slicing the multi-MB REL per entry is slow.
                delta, kind, section, addend = struct.unpack_from(">HBBI", rel, at)
                at += 8
                if kind == R_DOLPHIN_END:
                    break
                if kind == R_DOLPHIN_SECTION:
                    base = self.rel_sections[section]
                elif kind == R_DOLPHIN_NOP:
                    base = None if base is None else base + delta
                else:
                    base = (base or 0) + delta
                    if kind == R_PPC_ADDR32 and base:
                        if module == self.rel_id:
                            so = self.rel_sections[section]
                            out[base] = ("rel", so + addend) if so else ("bss", addend)
                        elif module == 0:
                            o = self.dol_offset(addend)
                            out[base] = ("dol", o) if o is not None else ("addr", addend)
        return out


# foresta.sym section letters -> REL section index (disc/src/assets.rs).
REL_SECTION = {"r": 4, "d": 5}


def load_symbols(rom):
    """[(name, src, offset, size)] from disc/data/foresta.sym (values
    relative to REL rodata/data, made file offsets here) and dol.sym
    (addresses; bss has no bytes and is skipped)."""
    syms = []
    for fname, src in (("foresta.sym", "rel"), ("dol.sym", "dol")):
        path = os.path.join(SYM_DIR, fname)
        if not os.path.exists(path):
            continue
        with open(path, encoding="utf-8", errors="replace") as f:
            for line in f:
                parts = line.split("\t")
                if len(parts) < 4:
                    continue
                name, sec, value, size = parts[0], parts[1], int(parts[2], 16), int(parts[3], 16)
                if src == "rel":
                    section = REL_SECTION.get(sec)
                    if section is None or section >= len(rom.rel_sections) or not rom.rel_sections[section]:
                        continue
                    syms.append((name, "rel", rom.rel_sections[section] + value, size))
                elif sec != "b":
                    off = rom.dol_offset(value)
                    if off is not None:
                        syms.append((name, "dol", off, size))
    return syms


class SymIndex:
    def __init__(self, syms, rom):
        self.by_src = {"rel": [], "dol": []}
        for name, src, off, size in syms:
            self.by_src[src].append((off, size, name))
        for v in self.by_src.values():
            v.sort()

    def name(self, src, off):
        import bisect

        table = self.by_src.get(src, [])
        i = bisect.bisect_right(table, (off, float("inf"), "")) - 1
        if i >= 0:
            start, size, name = table[i]
            if off < start + max(size, 1):
                return name if off == start else "%s+0x%X" % (name, off - start)
        return "0x%X" % off


def hexdump(rom, index, src, offset, length):
    data = rom.rel if src == "rel" else rom.dol
    end = min(offset + length, len(data))
    for row in range(offset, end, 16):
        chunk = data[row : min(row + 16, end)]
        words = " ".join(chunk[i : i + 4].hex() for i in range(0, len(chunk), 4))
        notes = []
        if src == "rel":
            for w in range(row - row % 4, row + 16, 4):
                if w in rom.pointers and offset <= w < end:
                    tsrc, toff = rom.pointers[w]
                    notes.append("+%X->%s" % (w - row, index.name(tsrc, toff) if tsrc in ("rel", "dol") else "%s:0x%X" % (tsrc, toff)))
        print("  %06X  %-35s %s" % (row, words, " ".join(notes)))


def resolve_image(image):
    if image != "rom":
        return image
    env = os.environ.get("BEVY_CROSSING_ISO")
    if env:
        return env
    rom_dir = os.path.join(os.getcwd(), "rom")
    found = [
        os.path.join(rom_dir, n)
        for n in (os.listdir(rom_dir) if os.path.isdir(rom_dir) else [])
        if n.lower().endswith((".iso", ".gcm"))
    ]
    if len(found) != 1:
        raise IsoError("'rom': set BEVY_CROSSING_ISO or keep one .iso/.gcm in ./rom")
    return found[0]


def cmd_sym(args):
    with GcmImage(args.image) as img:
        rom = Rom(img)
    syms = load_symbols(rom)
    if not syms:
        raise IsoError("no symbol maps in %s" % os.path.normpath(SYM_DIR))
    index = SymIndex(syms, rom)
    pat = args.pattern.lower()
    hits = [s for s in syms if (s[0].lower() == pat if args.exact else pat in s[0].lower())]
    if not hits:
        raise IsoError("no symbol matches %s" % args.pattern)
    for name, src, off, size in hits[: args.max]:
        print("%s  %s 0x%X  size 0x%X" % (name, src, off, size))
        if args.x:
            hexdump(rom, index, src, off, min(size or args.x, args.x))
    if len(hits) > args.max:
        print("... %d more (narrow the pattern or raise --max)" % (len(hits) - args.max))


def cmd_hex(args):
    with GcmImage(args.image) as img:
        rom = Rom(img)
    index = SymIndex(load_symbols(rom), rom)
    off = int(args.offset, 0)
    print("%s 0x%X  %s" % (args.src, off, index.name(args.src, off)))
    hexdump(rom, index, args.src, off, args.n)


def cmd_arc(args):
    with GcmImage(args.image) as img:
        data, files = rarc_files(img.read_file(args.arc))
    if not args.member:
        for name, _off, size in files:
            print("%10d  %s" % (size, name))
        return
    for name, off, size in files:
        if name.lower() == args.member.lower():
            print("%s  offset 0x%X  size 0x%X" % (name, off, size))
            for row in range(0, min(size, args.x), 16):
                chunk = data[off + row : off + min(row + 16, size)]
                print("  %06X  %s" % (row, " ".join(chunk[i : i + 4].hex() for i in range(0, len(chunk), 4))))
            return
    raise IsoError("no member %s in %s" % (args.member, args.arc))


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

    p = sub.add_parser("sym", help="find symbols and hex-dump them (stdout only)")
    p.add_argument("image")
    p.add_argument("pattern", help="substring of the symbol name (case-insensitive)")
    p.add_argument("-x", type=int, default=0, metavar="N", help="hex-dump up to N bytes of each match")
    p.add_argument("--exact", action="store_true", help="whole-name match")
    p.add_argument("--max", type=int, default=20, help="matches shown (default 20)")
    p.set_defaults(func=cmd_sym)

    p = sub.add_parser("hex", help="hex-dump the REL or DOL at a file offset")
    p.add_argument("image")
    p.add_argument("src", choices=["rel", "dol"])
    p.add_argument("offset", help="file offset, e.g. 0x3bdc")
    p.add_argument("-n", type=int, default=64, help="bytes (default 64)")
    p.set_defaults(func=cmd_hex)

    p = sub.add_parser("arc", help="list a RARC archive, or dump one member")
    p.add_argument("image")
    p.add_argument("arc", help="archive on the disc, e.g. forest_1st.arc")
    p.add_argument("member", nargs="?")
    p.add_argument("-x", type=int, default=64, metavar="N", help="bytes of the member to dump (default 64)")
    p.set_defaults(func=cmd_arc)

    args = ap.parse_args(argv)
    try:
        args.image = resolve_image(args.image)
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
