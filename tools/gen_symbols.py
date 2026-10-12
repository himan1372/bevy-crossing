"""Generate disc/data/{foresta,dol}.sym from the ac-decomp symbol maps.

Usage: python tools/gen_symbols.py <ac-decomp>/config/GAFE01_00

Output is metadata only (names, sections, offsets, sizes) - no game bytes.
Keeps named data objects (.data/.rodata/.bss); skips code and anonymous
compiler symbols ('@123'). One line per symbol: name<TAB>sec<TAB>value<TAB>size
(hex). sec is one char: d=.data r=.rodata b=.bss s=.sdata t=.sdata2 o=other.
For foresta, value is section-relative; for dol it's the virtual address.
"""
import re
import sys
from pathlib import Path

LINE = re.compile(r"^(\S+) = (\.\w+):0x([0-9A-Fa-f]+); // type:object size:0x([0-9A-Fa-f]+)")
SEC = {".data": "d", ".rodata": "r", ".bss": "b", ".sdata": "s", ".sdata2": "t"}


def convert(src: Path, dst: Path) -> int:
    out = []
    for line in src.read_text(encoding="utf-8").splitlines():
        m = LINE.match(line)
        if not m or m.group(1).startswith("@"):
            continue
        name, sec, value, size = m.groups()
        if sec == ".text":
            continue
        out.append(f"{name}\t{SEC.get(sec, 'o')}\t{int(value, 16):x}\t{int(size, 16):x}")
    dst.write_text("\n".join(out) + "\n", encoding="utf-8", newline="\n")
    return len(out)


def main() -> None:
    config = Path(sys.argv[1])
    data = Path(__file__).resolve().parent.parent / "disc" / "data"
    data.mkdir(exist_ok=True)
    print("foresta:", convert(config / "foresta" / "symbols.txt", data / "foresta.sym"))
    print("dol:", convert(config / "symbols.txt", data / "dol.sym"))


if __name__ == "__main__":
    main()
