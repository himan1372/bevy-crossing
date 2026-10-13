#!/usr/bin/env python3
"""Tests for gc_iso_reader.py. Builds tiny synthetic GCM images in a temp dir;
no real game data is used or needed. Run: python test_gc_iso_reader.py"""

import os
import struct
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gc_iso_reader import GcmImage, IsoError, GC_MAGIC

READER = os.path.join(os.path.dirname(os.path.abspath(__file__),
                                      ), "gc_iso_reader.py")


def be32(v):
    return struct.pack(">I", v)


def build_image(path, game_id=b"GAFE01", corrupt_magic=False,
                truncate_fst=False):
    img = bytearray(0x6000)
    # Header.
    img[0x00:0x06] = game_id
    img[0x06:0x08] = b"01"
    img[0x08] = 0  # disc number
    img[0x09] = 0  # version
    img[0x1C:0x20] = be32(0xDEADBEEF if corrupt_magic else GC_MAGIC)
    name = b"TEST GAME"
    img[0x20:0x20 + len(name)] = name
    # boot.bin info.
    dol_off, fst_off = 0x1000, 0x2000
    img[0x420:0x424] = be32(dol_off)
    img[0x424:0x428] = be32(fst_off)
    # Minimal DOL: one text section at file 0x100, size 0x40.
    img[dol_off + 0x00:dol_off + 0x04] = be32(0x100)
    img[dol_off + 0x90:dol_off + 0x94] = be32(0x40)
    img[dol_off + 0x100:dol_off + 0x140] = b"\x7C\x08\x02\xA6" * 16
    # FST: root + opening.bnr + dir files/ + a.bin + b.bin + root.bin.
    names = b"opening.bnr\x00files\x00a.bin\x00b.bin\x00root.bin\x00"
    noffs = {"opening.bnr": 0, "files": 12, "a.bin": 18, "b.bin": 24,
             "root.bin": 30}

    def entry(flags, name, a, b):
        e = bytearray(12)
        off = noffs[name]
        e[0] = flags
        e[1], e[2], e[3] = (off >> 16) & 0xFF, (off >> 8) & 0xFF, off & 0xFF
        e[4:8] = be32(a)
        e[8:12] = be32(b)
        return e

    entries = [
        entry(0, "opening.bnr", 0, 6),          # 0 root, 6 entries
        entry(0, "opening.bnr", 0x5000, 0x20),  # 1
        entry(1, "files", 0, 5),                # 2 dir, next=5
        entry(0, "a.bin", 0x5020, 0x10),        # 3 files/a.bin
        entry(0, "b.bin", 0x5030, 0x08),        # 4 files/b.bin
        entry(0, "root.bin", 0x5040, 0x04),     # 5
    ]
    fst = b"".join(entries) + names
    if truncate_fst:
        fst = fst[:20]
    img[0x424:0x428] = be32(fst_off)
    img[0x428:0x42C] = be32(len(fst))
    img[fst_off:fst_off + len(fst)] = fst
    # File payloads.
    img[0x5000:0x5020] = bytes(range(0x20))
    img[0x5020:0x5030] = b"A" * 0x10
    img[0x5030:0x5038] = b"B" * 0x08
    img[0x5040:0x5044] = b"CD" * 2
    with open(path, "wb") as f:
        f.write(img)
    return path


class TestReader(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.iso = os.path.join(self.tmp.name, "test.gcm")
        build_image(self.iso)

    def tearDown(self):
        self.tmp.cleanup()

    def test_header(self):
        with GcmImage(self.iso) as img:
            self.assertEqual(img.game_id, "GAFE01")
            self.assertEqual(img.game_name, "TEST GAME")
            self.assertEqual(img.maker, "01")
            self.assertEqual(img.dol_offset, 0x1000)
            self.assertEqual(img.fst_offset, 0x2000)

    def test_file_list(self):
        with GcmImage(self.iso) as img:
            paths = sorted(p for p, _, _ in img.files)
            self.assertEqual(paths, ["files/a.bin", "files/b.bin",
                                     "opening.bnr", "root.bin"])
            sizes = {p: s for p, _, s in img.files}
            self.assertEqual(sizes["opening.bnr"], 0x20)
            self.assertEqual(sizes["files/a.bin"], 0x10)

    def test_extract_bytes(self):
        with GcmImage(self.iso) as img:
            self.assertEqual(img.read_file("opening.bnr"), bytes(range(0x20)))
            self.assertEqual(img.read_file("files/a.bin"), b"A" * 0x10)
            self.assertEqual(img.read_file("files/b.bin"), b"B" * 0x08)
            self.assertEqual(img.read_file("root.bin"), b"CD" * 2)
            # Case-insensitive, leading slash tolerated.
            self.assertEqual(img.read_file("/FILES/A.BIN"), b"A" * 0x10)

    def test_missing_file(self):
        with GcmImage(self.iso) as img:
            with self.assertRaises(IsoError):
                img.read_file("nope.bin")

    def test_dol(self):
        with GcmImage(self.iso) as img:
            dol = img.read_dol()
            self.assertEqual(len(dol), 0x140)
            self.assertEqual(dol[0x100:0x104], b"\x7C\x08\x02\xA6")

    def test_bad_magic(self):
        bad = os.path.join(self.tmp.name, "bad.gcm")
        build_image(bad, corrupt_magic=True)
        with self.assertRaises(IsoError):
            GcmImage(bad)

    def test_truncated_fst(self):
        bad = os.path.join(self.tmp.name, "trunc.gcm")
        build_image(bad, truncate_fst=True)
        with self.assertRaises(IsoError):
            GcmImage(bad)

    def test_cli_info(self):
        r = subprocess.run([sys.executable, READER, "info", self.iso],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("GAFE01", r.stdout)
        self.assertIn("TEST GAME", r.stdout)

    def test_cli_ls(self):
        r = subprocess.run([sys.executable, READER, "ls", self.iso],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("opening.bnr", r.stdout)
        self.assertIn("files/a.bin", r.stdout)

    def test_cli_ls_subdir(self):
        r = subprocess.run([sys.executable, READER, "ls", self.iso, "files"],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("files/a.bin", r.stdout)
        self.assertNotIn("opening.bnr", r.stdout)

    def test_cli_extract(self):
        out = os.path.join(self.tmp.name, "out", "a.bin")
        r = subprocess.run([sys.executable, READER, "extract", self.iso,
                            "files/a.bin", "-o", out],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        with open(out, "rb") as f:
            self.assertEqual(f.read(), b"A" * 0x10)

    def test_cli_extract_missing(self):
        out = os.path.join(self.tmp.name, "nope.bin")
        r = subprocess.run([sys.executable, READER, "extract", self.iso,
                            "nope.bin", "-o", out],
                           capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)
        self.assertIn("not found", r.stderr)

    def test_cli_dol(self):
        out = os.path.join(self.tmp.name, "main.dol")
        r = subprocess.run([sys.executable, READER, "dol", self.iso,
                            "-o", out],
                           capture_output=True, text=True)
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(os.path.getsize(out), 0x140)


if __name__ == "__main__":
    unittest.main(verbosity=2)
