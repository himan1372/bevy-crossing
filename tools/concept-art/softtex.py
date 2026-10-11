"""Soft, hand-painted-looking procedural textures (v2 style: smooth, pastel,
flat colour with a faint brush grain). All painted from code."""
import math

import numpy as np

from tex import _grid, blank, hexrgb, stamp_polygon, star

RNG = np.random.default_rng(11)


def smooth_noise(size, cells, seed=0):
    rng = np.random.default_rng(seed)
    g = rng.random((cells + 1, cells + 1)).astype(np.float32)
    g[-1, :] = g[0, :]
    g[:, -1] = g[:, 0]
    t = np.linspace(0, cells, size, endpoint=False)
    i = t.astype(int)
    f = t - i
    f = f * f * (3 - 2 * f)
    a = g[i][:, i]
    b = g[i][:, i + 1]
    c = g[i + 1][:, i]
    d = g[i + 1][:, i + 1]
    fx = f[None, :]
    fy = f[:, None]
    return (a * (1 - fx) + b * fx) * (1 - fy) + (c * (1 - fx) + d * fx) * fy


def brush(arr, amt=0.05, seed=1):
    h, w = arr.shape[:2]
    n = smooth_noise(h, 6, seed) * 0.6 + smooth_noise(h, 24, seed + 1) * 0.4 - 0.5
    if w != h:
        n = np.resize(n, (h, w))
    arr[..., :3] = np.clip(arr[..., :3] * (1 + amt * 2 * n)[..., None], 0, 1)
    return arr


def blend(arr, m, color, a=1.0):
    m = (m * a)[..., None]
    arr[..., :3] = arr[..., :3] * (1 - m) + hexrgb(color)[:3] * m


def ell(x, y, cx, cy, rx, ry):
    """Antialiased filled ellipse mask (pixel units) plus normalised distance."""
    d = np.sqrt(((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2)
    return np.clip((1 - d) * min(rx, ry) / 1.2, 0, 1), d


def soft_face(fur, size=1024, eyes=True, sclera=False, iris="#3a2622", lid=0.0,
              muzzle=None, nose="#4a2f2a", mouth="smile", blush=True,
              eye_y=0.47, eye_dx=0.135, eye_rx=0.06, eye_ry=0.075, lash="#2b1d1a"):
    """Whole-head texture: fur colour everywhere, face features in the middle
    (the head's front is planar-projected onto this square)."""
    arr = blank(size, size, fur)
    x, y = _grid(size, size)
    S = size
    if muzzle:
        m, _ = ell(x, y, 0.5 * S, 0.6 * S, 0.16 * S, 0.105 * S)
        blend(arr, m, muzzle)
    if eyes:
        sides = (-1, 1) if eye_dx else (0,)
        for s in sides:
            cx, cy = (0.5 + s * eye_dx) * S, eye_y * S
            rx, ry = eye_rx * S, eye_ry * S
            outer, d = ell(x, y, cx, cy, rx, ry)
            if sclera:
                blend(arr, outer, "#ffffff")
                m, _ = ell(x, y, cx - s * rx * 0.08, cy + ry * 0.08, rx * 0.72, ry * 0.78)
                blend(arr, m, iris)
                m, _ = ell(x, y, cx - s * rx * 0.08, cy + ry * 0.1, rx * 0.36, ry * 0.4)
                blend(arr, m, lash)
            else:
                blend(arr, outer, iris)
            m, _ = ell(x, y, cx - rx * 0.28, cy - ry * 0.3, rx * 0.3, rx * 0.3)
            blend(arr, m, "#ffffff")
            m, _ = ell(x, y, cx + rx * 0.3, cy + ry * 0.35, rx * 0.13, rx * 0.13)
            blend(arr, m, "#ffffff")
            if lid > 0:
                ly = cy - ry + 2 * ry * lid
                cut = (y < ly) & (d < 1.06)
                arr[cut, :3] = hexrgb(fur)[:3]
                line = (np.abs(y - ly) < S * 0.007) & (d < 1.1)
                arr[line, :3] = hexrgb(lash)[:3]
            else:
                arc = (d > 0.98) & (d < 1.16) & (y < cy - ry * 0.15)
                arr[arc, :3] = hexrgb(lash)[:3]
    if nose:
        m, _ = ell(x, y, 0.5 * S, 0.565 * S, 0.034 * S, 0.022 * S)
        blend(arr, m, nose)
        m, _ = ell(x, y, 0.49 * S, 0.559 * S, 0.01 * S, 0.006 * S)
        blend(arr, m, "#ffffff", 0.7)
    t = S * 0.0075

    def ring(cx, cy, r, below):
        d = np.sqrt((x - cx) ** 2 + (y - cy) ** 2)
        return (np.abs(d - r) < t) & (y > below)

    if mouth == "smile":
        arr[ring(0.5 * S, 0.575 * S, 0.05 * S, 0.6 * S), :3] = hexrgb("#6b3328")[:3]
    elif mouth == "w":
        for s in (-1, 1):
            arr[ring((0.5 + s * 0.026) * S, 0.588 * S, 0.026 * S, 0.592 * S), :3] = hexrgb("#6b3328")[:3]
    elif mouth == "wide":
        arr[ring(0.5 * S, 0.43 * S, 0.2 * S, 0.6 * S), :3] = hexrgb("#3f6b2a")[:3]
    if blush:
        for s in (-1, 1):
            m, _ = ell(x, y, (0.5 + s * 0.235) * S, 0.585 * S, 0.05 * S, 0.03 * S)
            blend(arr, m, "#ff8f8f", 0.65)
    return brush(arr, 0.03, seed=5)


def soft_grass(size=512, base="#7cc35f", tufts=("#8bd06c", "#6fb655", "#97d97a"), count=420):
    arr = blank(size, size, base)
    for _ in range(count):
        cx, cy = RNG.uniform(0, size, 2)
        r = RNG.uniform(size * 0.018, size * 0.034)
        stamp_polygon(arr, star(cx, cy, r, r * 0.5, int(RNG.integers(4, 6)), RNG.uniform(0, 6.3)),
                      hexrgb(tufts[RNG.integers(len(tufts))]))
    return brush(arr, 0.07, seed=2)


def soft_leaves(size=512, base="#3d8a49", cols=("#3b8746", "#4ca356", "#5ab463", "#6cc471"), count=420):
    arr = blank(size, size, base)
    for k in range(count):
        cx, cy = RNG.uniform(0, size, 2)
        L = RNG.uniform(size * 0.06, size * 0.1)
        W = L * 0.5
        a = RNG.uniform(0, math.pi * 2)
        ca, sa = math.cos(a), math.sin(a)
        pts = [((t - 0.5) * L, math.sin(math.pi * t) * W / 2) for t in np.linspace(0, 1, 9)]
        pts += [((t - 0.5) * L, -math.sin(math.pi * t) * W / 2) for t in np.linspace(1, 0, 9)]
        pts = [(cx + px * ca - py * sa, cy + px * sa + py * ca) for px, py in pts]
        # Later leaves land on top, so they get the lighter colours.
        idx = int(np.clip(k / count * len(cols) + RNG.uniform(-0.6, 0.6), 0, len(cols) - 1))
        stamp_polygon(arr, pts, hexrgb(cols[idx]))
    return brush(arr, 0.06, seed=3)


def soft_bark(w=256, h=512):
    arr = blank(w, h, "#a8723f")
    x, y = _grid(w, h)
    for _ in range(22):
        x0 = RNG.uniform(0, w)
        wd = RNG.uniform(3, 9)
        col = "#93602f" if RNG.random() < 0.6 else "#ba8550"
        xx = x0 + 6 * np.sin(y / h * math.pi * 2 * int(RNG.integers(1, 3)) + RNG.uniform(0, 6))
        dx = np.minimum(np.abs(x - xx), w - np.abs(x - xx))
        blend(arr, np.clip(1 - dx / wd, 0, 1), col, 0.8)
    return brush(arr, 0.05, seed=4)


def soft_shingles(w=256, h=256, base="#d9574e", line="#b8433c"):
    arr = blank(w, h, base)
    x, y = _grid(w, h)
    row = 32
    for r in range(h // row):
        y0 = r * row
        off = (row // 2) * (r % 2)
        for c in range(-1, w // row + 1):
            cx = c * row + off + row / 2
            d = np.sqrt(((x - cx) / (row / 2)) ** 2 + ((y - (y0 + row * 0.5)) / (row * 0.55)) ** 2)
            band = (d > 0.9) & (d < 1.1) & (y > y0 + row * 0.5) & (y < y0 + row * 1.1)
            arr[band, :3] = hexrgb(line)[:3]
    return brush(arr, 0.06, seed=6)


def soft_wall(w=256, h=256, base="#f6eedc", line="#e3d5b8"):
    arr = blank(w, h, base)
    for yy in range(0, h, 32):
        arr[yy:yy + 3, :, :3] = hexrgb(line)[:3]
    return brush(arr, 0.04, seed=7)


def soft_wood(w=256, h=256, base="#c08a55", line="#9c6b3e"):
    arr = blank(w, h, base)
    for xx in range(0, w, 64):
        arr[:, xx:xx + 4, :3] = hexrgb(line)[:3]
    return brush(arr, 0.08, seed=8)


def soft_window(lit=False, size=128):
    arr = blank(size, size, "#ffe08a" if lit else "#bfe0f5")
    x, y = _grid(size, size)
    if not lit:
        stripe = (np.abs((x + y) - size * 0.7) < size * 0.08).astype(np.float32)
        blend(arr, stripe, "#ffffff", 0.6)
    f = size // 10
    white = hexrgb("#ffffff")[:3]
    arr[:f, :, :3] = white
    arr[-f:, :, :3] = white
    arr[:, :f, :3] = white
    arr[:, -f:, :3] = white
    c = size // 2
    arr[c - f // 2: c + f // 2, :, :3] = white
    arr[:, c - f // 2: c + f // 2, :3] = white
    return arr


def soft_pattern(base, accent, kind="check", size=256, cell=32):
    arr = blank(size, size, base)
    x, y = _grid(size, size)
    if kind == "check":
        arr[((x // cell + y // cell) % 2) == 0, :3] = hexrgb(accent)[:3]
    elif kind == "stripe":
        arr[(y // cell) % 2 == 0, :3] = hexrgb(accent)[:3]
    elif kind == "dots":
        yy = (y + (x // cell % 2) * cell / 2) % cell
        d = np.sqrt(((x % cell) - cell / 2) ** 2 + (yy - cell / 2) ** 2)
        blend(arr, np.clip(cell * 0.28 - d, 0, 1), accent)
    return brush(arr, 0.05, seed=9)
