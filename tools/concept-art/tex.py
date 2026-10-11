"""Procedural, original textures for the concept art (numpy -> Blender images).

Everything here is painted from code: no reference images are sampled.
All arrays are float RGBA, shape (h, w, 4), origin at top-left; `to_image`
flips to Blender's bottom-left order.
"""
import math
import numpy as np
import bpy

RNG = np.random.default_rng(7)


def hexrgb(h, a=1.0):
    h = h.lstrip("#")
    return np.array([int(h[i:i + 2], 16) / 255.0 for i in (0, 2, 4)] + [a], dtype=np.float32)


def blank(w, h, color):
    img = np.zeros((h, w, 4), dtype=np.float32)
    img[:] = hexrgb(color) if isinstance(color, str) else color
    return img


def to_image(name, arr):
    h, w, _ = arr.shape
    img = bpy.data.images.get(name) or bpy.data.images.new(name, w, h, alpha=True)
    if img.size[0] != w or img.size[1] != h:
        img.scale(w, h)
    img.pixels.foreach_set(np.ascontiguousarray(arr[::-1]).ravel())
    img.pack()
    return img


def _grid(w, h):
    y, x = np.mgrid[0:h, 0:w].astype(np.float32)
    return x, y


def stamp_polygon(arr, pts, color):
    """Fill a polygon (list of (x, y)) with wraparound for tiling."""
    h, w, _ = arr.shape
    pts = np.array(pts, dtype=np.float32)
    x0, y0 = np.floor(pts.min(0)).astype(int)
    x1, y1 = np.ceil(pts.max(0)).astype(int)
    ys, xs = np.mgrid[y0:y1 + 1, x0:x1 + 1]
    px, py = xs + 0.5, ys + 0.5
    inside = np.zeros(px.shape, bool)
    n = len(pts)
    j = n - 1
    for i in range(n):
        xi, yi = pts[i]
        xj, yj = pts[j]
        cond = ((yi > py) != (yj > py)) & (px < (xj - xi) * (py - yi) / (yj - yi + 1e-9) + xi)
        inside ^= cond
        j = i
    arr[ys[inside] % h, xs[inside] % w] = color


def star(cx, cy, r_out, r_in, points, rot):
    pts = []
    for k in range(points * 2):
        r = r_out if k % 2 == 0 else r_in
        a = rot + k * math.pi / points
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def grass(size=512, dark="#2f7d32", mids=("#3f9a3a", "#4caf3f", "#5cbf48"), count=520):
    """Carpet of star/leaf tufts in a few greens over a darker base."""
    arr = blank(size, size, dark)
    for _ in range(count):
        cx, cy = RNG.uniform(0, size, 2)
        r = RNG.uniform(size * 0.028, size * 0.05)
        col = hexrgb(mids[RNG.integers(len(mids))])
        stamp_polygon(arr, star(cx, cy, r, r * 0.45, int(RNG.integers(4, 6)), RNG.uniform(0, 6.3)), col)
    return arr


def dirt_patch(size=256):
    """Tan oval with concentric swirl rings, alpha outside the oval."""
    x, y = _grid(size, size)
    u = (x - size / 2) / (size / 2)
    v = (y - size / 2) / (size / 2 * 0.72)
    d = np.sqrt(u * u + v * v)
    ang = np.arctan2(v, u)
    rings = np.sin((d * 7.0 + 0.18 * np.sin(ang * 3.0)) * math.pi)
    arr = blank(size, size, "#d9b45a")
    arr[rings > 0.25] = hexrgb("#e9cc78")
    arr[rings < -0.55] = hexrgb("#c49a43")
    arr[..., 3] = (d < 1.0).astype(np.float32)
    return arr


def cobbles(size=512, cells=5):
    """Chunky blue-grey cobbles with dark outlines (tileable)."""
    x, y = _grid(size, size)
    s = size / cells
    pts = []
    for gy in range(cells):
        for gx in range(cells):
            jx, jy = RNG.uniform(-0.18, 0.18, 2)
            pts.append(((gx + 0.5 + jx + 0.5 * (gy % 2)) * s % size, (gy + 0.5 + jy) * s))
    best = np.full(x.shape, 1e9, np.float32)
    second = np.full(x.shape, 1e9, np.float32)
    idx = np.zeros(x.shape, np.int32)
    for k, (cx, cy) in enumerate(pts):
        dx = np.minimum(np.abs(x - cx), size - np.abs(x - cx))
        dy = np.minimum(np.abs(y - cy), size - np.abs(y - cy))
        d = np.maximum(np.maximum(dx, dy), (dx + dy) * 0.72)  # octagon-ish
        closer = d < best
        second = np.where(closer, best, np.minimum(second, d))
        best = np.where(closer, d, best)
        idx = np.where(closer, k, idx)
    shades = [hexrgb(c) for c in ("#9fb3cf", "#a9bcd6", "#94a9c6", "#b2c3da")]
    arr = blank(size, size, "#9fb3cf")
    for i in range(len(shades)):
        arr[idx % len(shades) == i] = shades[i]
    gap = second - best
    arr[gap < s * 0.16] = hexrgb("#7d93b3")
    arr[gap < s * 0.07] = hexrgb("#4a5f80")
    return arr


def cliff(w=256, h=256):
    """Vertical bands of tan, cream and brown with darker seams."""
    arr = blank(w, h, "#c9a26b")
    x = 0
    palette = [hexrgb(c) for c in ("#e8d3a4", "#c9a26b", "#a77b4c", "#d9bb84")]
    while x < w:
        bw = int(RNG.integers(10, 30))
        arr[:, x:x + bw] = palette[RNG.integers(len(palette))]
        arr[:, x:x + 2] = hexrgb("#7a5532")
        x += bw
    # A few horizontal ledges.
    for yy in RNG.integers(20, h - 20, 3):
        arr[yy:yy + 2, :] = hexrgb("#8a6440")
    return arr


def water(size=512, cells=12, deep="#1c45b0"):
    """Deep blue with a white web of ripples (Voronoi edges)."""
    x, y = _grid(size, size)
    pts = RNG.uniform(0, size, (cells * cells // 3, 2))
    d1 = np.full(x.shape, 1e9, np.float32)
    d2 = np.full(x.shape, 1e9, np.float32)
    for px, py in pts:
        dx = np.minimum(np.abs(x - px), size - np.abs(x - px))
        dy = np.minimum(np.abs(y - py), size - np.abs(y - py))
        d = np.sqrt(dx * dx + dy * dy)
        nd1 = np.minimum(d1, d)
        d2 = np.where(d < d1, d1, np.minimum(d2, d))
        d1 = nd1
    edge = d2 - d1
    arr = blank(size, size, deep)
    arr[edge < 6] = hexrgb("#2d5fd0")
    arr[edge < 2.0] = hexrgb("#b8d4ff")
    return arr


def waterfall(w=128, h=256):
    arr = blank(w, h, "#5c8fe8")
    for xx in range(0, w, 6):
        shade = "#d6e8ff" if RNG.random() < 0.4 else "#8db4f2"
        arr[:, xx:xx + int(RNG.integers(2, 5))] = hexrgb(shade)
    return arr


def bark(w=128, h=256):
    arr = blank(w, h, "#8a5a32")
    x, y = _grid(w, h)
    for _ in range(7):
        cx, cy = RNG.uniform(0, w), RNG.uniform(0, h)
        d = np.sqrt(((x - cx) / 10) ** 2 + ((y - cy) / 20) ** 2)
        arr[(d > 0.6) & (d < 1.0)] = hexrgb("#6b4223")
        arr[d < 0.6] = hexrgb("#a8774a")
    for xx in range(0, w, 16):
        arr[:, xx:xx + 2] = hexrgb("#6b4223")
    return arr


def leaves(size=256):
    """Clusters of chunky leaf lobes in teal-greens."""
    return grass(size, dark="#1f5a3a", mids=("#2e7a4a", "#3a8f55", "#257048"), count=180)


def planks(w=256, h=256, base="#c4413a", line="#8e2823", vertical=False):
    arr = blank(w, h, base)
    step = 32
    if vertical:
        for xx in range(0, w, step):
            arr[:, xx:xx + 3] = hexrgb(line)
    else:
        for yy in range(0, h, step):
            arr[yy:yy + 3, :] = hexrgb(line)
    return arr


def wall(w=256, h=256):
    arr = planks(w, h, base="#f2e6c9", line="#cdbb95", vertical=True)
    return arr


def blob_shadow(size=128):
    x, y = _grid(size, size)
    d = np.sqrt((x - size / 2) ** 2 + (y - size / 2) ** 2) / (size / 2)
    arr = blank(size, size, "#000000")
    arr[..., 3] = np.clip(1.0 - d, 0, 1) ** 0.8 * 0.55
    return arr


def face(skin=None, eye="#2a2a2a", blush=True, nose="#e8743a", size=256, eyes_up=0.0):
    """Painted face features (big round eyes, small nose, optional blush) on a
    transparent background, to overlay a head."""
    arr = np.zeros((size, size, 4), dtype=np.float32)
    if skin:
        arr[:] = hexrgb(skin)
    x, y = _grid(size, size)
    ey = size * (0.45 - eyes_up)
    for ex in (size * 0.33, size * 0.67):
        d = np.sqrt((x - ex) ** 2 + ((y - ey) * 0.85) ** 2)
        arr[d < size * 0.11] = hexrgb("#ffffff")
        arr[d < size * 0.075] = hexrgb(eye)
        hl = np.sqrt((x - ex + size * 0.025) ** 2 + (y - ey + size * 0.03) ** 2)
        arr[hl < size * 0.022] = hexrgb("#ffffff")
    if nose:
        stamp_polygon(arr, [(size * 0.5, size * 0.55), (size * 0.45, size * 0.66), (size * 0.55, size * 0.66)], hexrgb(nose))
    mouth = (np.abs(np.sqrt((x - size * 0.5) ** 2 + (y - size * 0.62) ** 2) - size * 0.1) < 3) & (y > size * 0.68)
    arr[mouth] = hexrgb("#7a3b2a")
    if blush:
        for bx in (size * 0.2, size * 0.8):
            d = np.sqrt((x - bx) ** 2 + ((y - size * 0.6) * 1.6) ** 2)
            arr[d < size * 0.06] = hexrgb("#ff9a9a", 0.75)
    return arr


def pattern(base, accent, kind="check", size=128, cell=16):
    arr = blank(size, size, base)
    x, y = _grid(size, size)
    if kind == "check":
        arr[((x // cell + y // cell) % 2) == 0] = hexrgb(accent)
    elif kind == "stripe":
        arr[(y // cell) % 2 == 0] = hexrgb(accent)
    elif kind == "dots":
        d = np.sqrt(((x % cell) - cell / 2) ** 2 + ((y % cell) - cell / 2) ** 2)
        arr[d < cell * 0.28] = hexrgb(accent)
    return arr


def window_glow(lit=True):
    arr = blank(64, 64, "#ffe28a" if lit else "#5a7aa8")
    arr[30:34, :] = hexrgb("#6b4223")
    arr[:, 30:34] = hexrgb("#6b4223")
    arr[:3, :] = arr[-3:, :] = hexrgb("#6b4223")
    arr[:, :3] = arr[:, -3:] = hexrgb("#6b4223")
    return arr
