"""Concept art v2: soft, rounded, pastel style (Philip's anchor: "more like
Pots the Raccoon"). All geometry and textures are original and generated here.

Run:  blender -b --factory-startup --python build2.py -- <scene> <out.png> [width height]
Scenes: square, river, lineup, night, wide, chat
"""
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(__file__))
import softtex as st  # noqa: E402
import tex  # noqa: E402

NIGHT = False

# ------------------------------------------------------------------ setup


def reset():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    for eng in ("BLENDER_EEVEE", "BLENDER_EEVEE_NEXT"):
        try:
            sc.render.engine = eng
            break
        except TypeError:
            continue
    sc.view_settings.view_transform = "Standard"
    sc.view_settings.look = "None"
    try:
        sc.eevee.taa_render_samples = 64
    except AttributeError:
        pass
    return sc


def lin(c):
    """sRGB 0..1 -> linear (for flat colours fed to shader sockets)."""
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def rgb(hexstr):
    r, g, b, _ = tex.hexrgb(hexstr)
    return (lin(r), lin(g), lin(b), 1.0)


MC = {}


def smat(name, color=None, arr=None, scale=1.0, emit=0.3, alpha=False, extend=False):
    """Soft matte material: base colour plus a little self-light so shading
    stays gentle (the reference models are lit mostly by emission)."""
    if name in MC:
        return MC[name]
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    b = nt.nodes["Principled BSDF"]
    b.inputs["Roughness"].default_value = 1.0
    for k in ("Specular IOR Level", "Specular"):
        if k in b.inputs:
            b.inputs[k].default_value = 0.0
            break
    if arr is not None:
        t = nt.nodes.new("ShaderNodeTexImage")
        t.image = tex.to_image(name, arr)
        t.interpolation = "Linear"
        t.extension = "EXTEND" if extend else "REPEAT"
        if scale != 1.0:
            uv = nt.nodes.new("ShaderNodeTexCoord")
            mp = nt.nodes.new("ShaderNodeMapping")
            mp.inputs["Scale"].default_value = (scale, scale, 1)
            nt.links.new(uv.outputs["UV"], mp.inputs["Vector"])
            nt.links.new(mp.outputs["Vector"], t.inputs["Vector"])
        col = t.outputs["Color"]
        if alpha:
            nt.links.new(t.outputs["Alpha"], b.inputs["Alpha"])
            if hasattr(m, "surface_render_method"):
                m.surface_render_method = "BLENDED"
    else:
        node = nt.nodes.new("ShaderNodeRGB")
        node.outputs[0].default_value = rgb(color)
        col = node.outputs[0]
    nt.links.new(col, b.inputs["Base Color"])
    if emit:
        nt.links.new(col, b.inputs["Emission Color"])
        b.inputs["Emission Strength"].default_value = emit * (0.35 if NIGHT else 1.0)
    MC[name] = m
    return m


def flat(color, emit=0.3):
    return smat("c_" + color.lstrip("#") + f"_{emit}", color=color, emit=emit)


def smooth(o, subsurf=0):
    for p in o.data.polygons:
        p.use_smooth = True
    if subsurf:
        m = o.modifiers.new("sub", "SUBSURF")
        m.levels = subsurf
        m.render_levels = subsurf
    return o


def sphere(loc, scale, mat, seg=32, ring=16, shape=None, rot=(0, 0, 0)):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=seg, ring_count=ring, radius=1, location=loc, rotation=rot)
    o = bpy.context.active_object
    if shape:
        for v in o.data.vertices:
            v.co = shape(*v.co)
    o.scale = scale if isinstance(scale, (tuple, list)) else (scale,) * 3
    o.data.materials.append(mat)
    return smooth(o)


def rbox(loc, size, mat, bev=0.15, seg=4, rot=(0, 0, 0)):
    """Rounded box. size = half extents."""
    bpy.ops.mesh.primitive_cube_add(size=2, location=loc, rotation=rot)
    o = bpy.context.active_object
    o.scale = size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    m = o.modifiers.new("bevel", "BEVEL")
    m.width = bev
    m.segments = seg
    m.limit_method = "NONE"
    m.harden_normals = True
    o.data.materials.append(mat)
    return smooth(o)


def cyl(loc, r, depth, mat, rot=(0, 0, 0), verts=24, bev=0.0):
    bpy.ops.mesh.primitive_cylinder_add(vertices=verts, radius=r, depth=depth, location=loc, rotation=rot)
    o = bpy.context.active_object
    if bev:
        m = o.modifiers.new("bevel", "BEVEL")
        m.width = bev
        m.segments = 3
        m.limit_method = "ANGLE"
        m.harden_normals = True
    o.data.materials.append(mat)
    return smooth(o)


def cone(loc, r1, r2, depth, mat, rot=(0, 0, 0), verts=24):
    bpy.ops.mesh.primitive_cone_add(vertices=verts, radius1=r1, radius2=r2, depth=depth, location=loc, rotation=rot)
    o = bpy.context.active_object
    o.data.materials.append(mat)
    return smooth(o, 1)


def plane(w, h, loc, mat, rot=(0, 0, 0), subdiv=0):
    if subdiv:
        bpy.ops.mesh.primitive_grid_add(x_subdivisions=subdiv, y_subdivisions=subdiv, size=1, location=loc, rotation=rot)
    else:
        bpy.ops.mesh.primitive_plane_add(size=1, location=loc, rotation=rot)
    o = bpy.context.active_object
    o.scale = (w, h, 1)
    o.data.materials.append(mat)
    return o


def mesh_obj(name, verts, faces, mat, uvs=None):
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    me.update()
    if uvs is not None:
        uvl = me.uv_layers.new()
        for poly in me.polygons:
            for li in poly.loop_indices:
                uvl.data[li].uv = uvs[me.loops[li].vertex_index]
    o = bpy.data.objects.new(name, me)
    bpy.context.collection.objects.link(o)
    me.materials.append(mat)
    return o


# ------------------------------------------------------------------ materials

M = {}


def build_mats():
    M["grass"] = smat("grass", arr=st.soft_grass(), scale=18)
    M["cobble"] = smat("cobble", arr=tex.cobbles(shades=("#c9cfe3", "#bfc6dc", "#d3d8ea", "#b8c0d8"), mid="#a3abc6", line="#8a92b2"), scale=1)
    M["dirt"] = smat("dirt", arr=st.brush(tex.dirt_patch(base="#e2c27c", light="#eed69c", dark="#d3b068"), 0.05), alpha=True)
    M["cliff"] = smat("cliff", arr=st.brush(tex.cliff(palette=("#f1e0bb", "#dfc497", "#cdab7c", "#e8d2a6"), seam="#b8956a", ledge="#ae8a60"), 0.05), scale=2)
    M["water"] = smat("water", arr=st.brush(tex.water(deep="#4e98e3", mid="#5fa9ee", line="#dcefff", mid_w=7, line_w=2.5), 0.04), scale=3, emit=0.45)
    M["fall"] = smat("fall", arr=tex.waterfall(base="#7fb6f0", light="#eef7ff", mid="#a8cef6"), emit=0.6)
    M["bark"] = smat("bark", arr=st.soft_bark(), scale=1.0)
    M["leaves"] = smat("leaves", arr=st.soft_leaves(), scale=1.5)
    M["roof"] = smat("roof", arr=st.soft_shingles(), scale=2)
    M["roof_blue"] = smat("roof_blue", arr=st.soft_shingles(base="#5b8fd6", line="#4371b5"), scale=2)
    M["roof_green"] = smat("roof_green", arr=st.soft_shingles(base="#6cb37a", line="#518f5e"), scale=2)
    M["wall"] = smat("wall", arr=st.soft_wall(), scale=1.5)
    M["wood"] = smat("wood", arr=st.soft_wood(), scale=1.0)
    M["shadow"] = smat("shadow", arr=tex.blob_shadow(), alpha=True, emit=0)
    M["white"] = flat("#fbf8f1")
    M["door"] = flat("#b9774a")
    M["fringe"] = flat("#74bd59")


def shadow(x, y, r, strength=1.0):
    o = plane(r * 2, r * 2, (x, y, 0.045), M["shadow"], subdiv=6)
    return o


# ------------------------------------------------------------------ nature


def tree(x, y, s=1.0, fruit=None):
    # Lathe trunk with a flared, lumpy root base.
    rings, around = 22, 28
    H = 2.0 * s
    verts, faces, uvs = [], [], []
    for i in range(rings + 1):
        t = i / rings
        z = t * H
        for j in range(around + 1):
            a = j / around * math.tau
            r = (0.3 + 0.32 * math.exp(-t * 7)) * s
            r *= 1 + 0.35 * max(0.0, math.cos(4 * a + 0.4)) * math.exp(-t * 9)
            r *= 1 - 0.12 * t
            verts.append((x + r * math.cos(a), y + r * math.sin(a), z))
            uvs.append((j / around * 2, t * 2))
    for i in range(rings):
        for j in range(around):
            a = i * (around + 1) + j
            b = a + around + 1
            faces.append((a, a + 1, b + 1, b))
    smooth(mesh_obj("trunk", verts, faces, M["bark"], uvs))
    lobes = [(0, 0, 2.55, 1.15), (0.85, 0.15, 2.3, 0.85), (-0.8, -0.1, 2.35, 0.9), (0.1, 0.75, 2.75, 0.85),
             (-0.25, -0.65, 2.7, 0.85), (0.35, -0.3, 3.2, 0.78), (-0.45, 0.35, 3.25, 0.7)]
    for dx, dy, dz, r in lobes:
        sphere((x + dx * s, y + dy * s, dz * s), (r * s, r * s, r * s * 0.85), M["leaves"], seg=24, ring=12)
    if fruit:
        fm = flat(fruit, 0.35)
        for dx, dy, dz in [(0.85, -0.7, 2.05), (-0.75, -0.75, 2.3), (0.25, -1.05, 2.75), (-0.2, -0.95, 1.95)]:
            sphere((x + dx * s, y + dy * s, dz * s), 0.17 * s, fm, seg=16, ring=8)
            cyl((x + dx * s, y + dy * s, dz * s + 0.17 * s), 0.02 * s, 0.1 * s, flat("#6b4a2b"), verts=6)
    shadow(x, y, 1.6 * s)


def bush(x, y, s=1.0, flower=None):
    for dx, dy, dz, r in [(0, 0, 0.45, 0.55), (0.45, 0.1, 0.35, 0.42), (-0.45, 0.05, 0.38, 0.45)]:
        sphere((x + dx * s, y + dy * s, dz * s), (r * s, r * s, r * s * 0.8), M["leaves"], seg=20, ring=10)
    if flower:
        fm = flat(flower, 0.4)
        for dx, dy, dz in [(0.2, -0.45, 0.6), (-0.35, -0.38, 0.5), (0.55, -0.25, 0.45), (-0.05, -0.3, 0.85)]:
            sphere((x + dx * s, y + dy * s, dz * s), 0.08 * s, fm, seg=10, ring=6)
    shadow(x, y, 1.0 * s)


def flowers(x, y, color, n=5, spread=0.6):
    pm = flat(color, 0.4)
    ym = flat("#ffd84d", 0.4)
    sm = flat("#4f9a46")
    for i in range(n):
        a = i * 2.39996
        r = spread * math.sqrt((i + 0.5) / n)
        fx, fy = x + r * math.cos(a), y + r * math.sin(a)
        cyl((fx, fy, 0.14), 0.022, 0.28, sm, verts=8)
        for k in range(5):
            pa = k * math.tau / 5
            sphere((fx + 0.085 * math.cos(pa), fy + 0.085 * math.sin(pa), 0.3), (0.07, 0.07, 0.035), pm, seg=12, ring=6)
        sphere((fx, fy, 0.31), (0.045, 0.045, 0.03), ym, seg=10, ring=6)
        sphere((fx + 0.07, fy, 0.08), (0.06, 0.03, 0.02), sm, seg=8, ring=4, rot=(0, 0, a))


def rock(x, y, s=1.0):
    sphere((x, y, 0.15 * s), (0.4 * s, 0.32 * s, 0.25 * s), flat("#b9b8c9"), seg=10, ring=6)


# ------------------------------------------------------------------ buildings


def house(x, y, roof="roof", lit=False):
    wall_h = 1.05
    rbox((x, y, wall_h), (1.35, 1.15, wall_h), M["wall"], bev=0.18)
    rbox((x, y, 0.08), (1.45, 1.25, 0.1), flat("#b7aea0"), bev=0.06)  # foundation
    a = math.radians(36)
    d = 1.5
    L = d / math.cos(a)
    top = 2 * wall_h
    for sgn in (-1, 1):
        rbox((x, y + sgn * d / 2, top + (d / 2) * math.tan(a)), (1.65, L / 2, 0.11), M[roof], bev=0.08,
             rot=(-sgn * a, 0, 0))
    apex = top + 1.15 * math.tan(a)
    for sx in (-1, 1):
        verts = [(x + sx * 1.3, y - 1.15, top - 0.01), (x + sx * 1.3, y + 1.15, top - 0.01), (x + sx * 1.3, y, apex)]
        mesh_obj("gable", verts, [(0, 1, 2)], M["wall"], [(0, 0), (2, 0), (1, 1)])
    cyl((x, y, top + d * math.tan(a) + 0.07), 0.13, 3.4, M[roof], rot=(0, math.pi / 2, 0), verts=16)
    # Arched door.
    door = M["door"]
    rbox((x, y - 1.16, 0.62), (0.36, 0.07, 0.5), door, bev=0.05)
    cyl((x, y - 1.16, 1.1), 0.36, 0.14, door, rot=(math.pi / 2, 0, 0), verts=24, bev=0.04)
    sphere((x + 0.22, y - 1.25, 0.72), 0.05, flat("#f0c24a", 0.4), seg=12, ring=8)
    wm = smat("win_lit" if lit else "win", arr=st.soft_window(lit), emit=4.0 if lit else 0.35)
    for wx in (-0.85, 0.85):
        rbox((x + wx, y - 1.16, 1.25), (0.3, 0.05, 0.3), M["white"], bev=0.06)
        plane(0.5, 0.5, (x + wx, y - 1.215, 1.25), wm, rot=(math.pi / 2, 0, 0))
        rbox((x + wx, y - 1.22, 0.88), (0.36, 0.09, 0.06), M["white"], bev=0.04)  # sill
    rbox((x + 0.85, y + 0.35, top + 1.05), (0.2, 0.2, 0.42), flat("#c46a55"), bev=0.06)
    rbox((x, y - 1.55, 0.06), (0.5, 0.3, 0.06), flat("#ddd6c8"), bev=0.05)
    # Mailbox.
    cyl((x + 1.75, y - 1.5, 0.45), 0.05, 0.9, M["wood"], verts=10)
    rbox((x + 1.75, y - 1.5, 1.0), (0.2, 0.28, 0.16), flat("#e7584c"), bev=0.1)
    shadow(x, y, 2.4)
    if lit:
        bpy.ops.object.light_add(type="POINT", location=(x, y - 1.8, 1.4))
        L_ = bpy.context.active_object
        L_.data.energy = 180
        L_.data.color = (1.0, 0.78, 0.45)
        L_.data.shadow_soft_size = 0.6


def fence(x0, y0, x1, y1, step=0.45):
    length = math.hypot(x1 - x0, y1 - y0)
    n = max(2, int(length / step))
    ang = math.atan2(y1 - y0, x1 - x0)
    w = M["white"]
    for i in range(n + 1):
        t = i / n
        px, py = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
        rbox((px, py, 0.4), (0.075, 0.05, 0.4), w, bev=0.04, rot=(0, 0, ang))
        sphere((px, py, 0.82), (0.08, 0.06, 0.08), w, seg=12, ring=6, rot=(0, 0, ang))
    for z in (0.3, 0.6):
        rbox(((x0 + x1) / 2, (y0 + y1) / 2, z), (length / 2, 0.035, 0.05), w, bev=0.03, rot=(0, 0, ang))


# ------------------------------------------------------------------ characters


def head(loc, scale, mat, cheeks=0.12):
    """Rounded head, a little wider at the cheeks. Its front half is planar-
    mapped onto the face texture; the back reads the texture's plain edge."""
    def shape(x, y, z):
        k = 1 + cheeks * max(0.0, -z) * 1.4
        return (x * k, y * (1 + cheeks * 0.6 * max(0.0, -z)), z * (1 - 0.06 * max(0.0, -z)))
    o = sphere(loc, scale, mat, seg=48, ring=24, shape=shape)
    me = o.data
    uvl = me.uv_layers.active
    for poly in me.polygons:
        for li in poly.loop_indices:
            x, y, z = me.vertices[me.loops[li].vertex_index].co
            v = min(1.0, max(0.0, 0.5 + z * 0.5))
            if y <= 0:
                u = min(1.0, max(0.0, 0.5 + x * 0.5))
            else:
                u = 0.0 if x < 0 else 1.0
            uvl.data[li].uv = (u, v)
    return o


def body(x, y, shirt, skin, legs, shoes, arms=0.95):
    def pear(px, py, pz):
        # Pear profile: narrow shoulders, wide hips, flat-ish bottom.
        r_now = max(1e-4, math.sqrt(max(0.0, 1 - pz * pz)))
        if pz >= 0:
            r_t = r_now * (1 - 0.62 * pz) * (1 - 0.15 * pz)
        else:
            r_t = (1 - abs(pz) ** 3.5) ** 0.3 * 0.9
        k = r_t / r_now
        return (px * k, py * k, pz)
    sphere((x, y, 0.58), (0.36, 0.3, 0.38), shirt, seg=40, ring=24, shape=pear)
    sk = flat(skin)
    lg = flat(legs)
    sh = flat(shoes)
    for s in (-1, 1):
        sphere((x + s * 0.13, y, 0.22), (0.1, 0.1, 0.16), lg, seg=16, ring=8)
        sphere((x + s * 0.13, y - 0.04, 0.06), (0.115, 0.15, 0.075), sh, seg=16, ring=8)
        sphere((x + s * 0.3, y, 0.8), (0.075, 0.075, 0.2), shirt, seg=16, ring=8, rot=(0, s * arms, 0))
        sphere((x + s * 0.45, y - 0.01, 0.66), 0.09, sk, seg=16, ring=8)


def kid(x, y, shirt, hat="#e86a6a", hair="#6b4630", skin="#f7d3b2"):
    """Original player kid: big round head, hair fringe, beanie with pom-pom."""
    body(x, y, shirt, skin, "#4f6fb8", "#e05a4a")
    face = smat("face_kid_" + hat, arr=st.soft_face(skin, sclera=False, iris="#3a2a26", nose="#e9a487", mouth="smile"),
                extend=True, emit=0.3)
    head((x, y, 1.38), (0.6, 0.55, 0.53), face)
    hm = flat(hair)
    sphere((x, y + 0.08, 1.5), (0.635, 0.6, 0.5), hm, seg=32, ring=16)
    for s in (-1, 1):
        sphere((x + s * 0.5, y - 0.05, 1.22), (0.12, 0.2, 0.22), hm, seg=16, ring=8)
    hb = flat(hat)
    sphere((x, y + 0.05, 1.74), (0.66, 0.62, 0.42), hb, seg=32, ring=16)
    sphere((x, y + 0.03, 1.58), (0.68, 0.64, 0.11), M["white"], seg=32, ring=10)
    sphere((x, y + 0.05, 2.17), 0.14, M["white"], seg=16, ring=10)
    shadow(x, y, 0.7)


VILLAGERS = {
    "cat": dict(fur="#f3b86a", face=dict(sclera=True, iris="#5aa35a", lid=0.38, muzzle="#fff4de", nose="#e8899a", mouth="w")),
    "bear": dict(fur="#bf9068", face=dict(sclera=False, muzzle="#f2dfbf", nose="#4a2f2a", mouth="smile")),
    "frog": dict(fur="#92d16c", face=dict(eyes=False, nose=None, mouth="wide")),
    "bird": dict(fur="#a2cbf0", face=dict(sclera=True, iris="#2e3f6b", nose=None, mouth=None, eye_dx=0.15)),
}


def villager(x, y, kind, shirt, legs=None):
    v = VILLAGERS[kind]
    fur = v["fur"]
    fm = flat(fur)
    body(x, y, shirt, fur, legs or fur, fur)
    face = smat(f"face_{kind}", arr=st.soft_face(fur, **v["face"]), extend=True)
    head((x, y, 1.38), (0.6, 0.54, 0.52), face)
    if kind == "cat":
        for s in (-1, 1):
            cone((x + s * 0.34, y + 0.05, 1.86), 0.19, 0.02, 0.42, fm, rot=(0, s * 0.35, 0), verts=16)
            cone((x + s * 0.33, y - 0.03, 1.84), 0.11, 0.01, 0.28, flat("#f7a7b0"), rot=(0.12, s * 0.35, 0), verts=12)
        for i in range(4):
            sphere((x + 0.25 + 0.06 * i, y + 0.32 + 0.02 * i, 0.4 + 0.12 * i), 0.07, fm, seg=12, ring=6)
    elif kind == "bear":
        for s in (-1, 1):
            sphere((x + s * 0.42, y + 0.05, 1.8), (0.17, 0.11, 0.17), fm, seg=16, ring=8)
            sphere((x + s * 0.42, y - 0.01, 1.8), (0.09, 0.06, 0.09), flat("#e9c9a3"), seg=12, ring=6)
        sphere((x, y + 0.3, 0.45), 0.09, fm, seg=12, ring=6)
    elif kind == "frog":
        eye = smat("frog_eye", arr=st.soft_face(fur, size=512, sclera=False, iris="#2a2a2a", nose=None, mouth=None,
                                                blush=False, eye_dx=0.0, eye_y=0.5, eye_rx=0.22, eye_ry=0.26), extend=True)
        for s in (-1, 1):
            head((x + s * 0.26, y - 0.08, 1.86), (0.19, 0.17, 0.18), eye, cheeks=0.0)
    elif kind == "bird":
        cone((x, y - 0.62, 1.3), 0.13, 0.0, 0.3, flat("#f5a83a", 0.35), rot=(math.pi / 2, 0, 0), verts=16)
        for i, a in enumerate((-0.3, 0.0, 0.3)):
            sphere((x + a * 0.25, y + 0.02, 1.95 + 0.03 * (i == 1)), (0.05, 0.05, 0.16), fm, seg=12, ring=6, rot=(0, a, 0))
    shadow(x, y, 0.7)


def pattern_mat(name, *a, **k):
    return smat("shirt_" + name, arr=st.soft_pattern(*a, **k), scale=1.5)


# ------------------------------------------------------------------ world, light, camera


def ground(size=70, subdiv=140):
    return plane(size, size, (0, 0, 0), M["grass"], subdiv=subdiv)


def dirt(x, y, w, h):
    plane(w, h, (x, y, 0.03), M["dirt"], subdiv=10)


def plaza(x, y, w, h):
    rbox((x, y, 0.0), (w / 2, h / 2, 0.04), M["cobble"], bev=0.03, seg=2)
    o = bpy.context.active_object
    # Re-map UVs to world-ish units so cobbles keep their size.
    me = o.data
    uvl = me.uv_layers.active
    for poly in me.polygons:
        for li in poly.loop_indices:
            co = me.vertices[me.loops[li].vertex_index].co
            uvl.data[li].uv = (co.x / 2.2, co.y / 2.2)
    o.modifiers.new("sub", "SUBSURF").levels = 0
    sub = o.modifiers.new("grid", "SUBSURF")
    sub.subdivision_type = "SIMPLE"
    sub.levels = sub.render_levels = 4


def world(day=True):
    w = bpy.data.worlds.new("world")
    bpy.context.scene.world = w
    w.use_nodes = True
    nt = w.node_tree
    out = nt.nodes["World Output"]
    bg = nt.nodes["Background"]
    sky = nt.nodes.new("ShaderNodeBackground")
    mix = nt.nodes.new("ShaderNodeMixShader")
    lp = nt.nodes.new("ShaderNodeLightPath")
    bg.inputs["Color"].default_value = (0.62, 0.72, 0.95, 1) if day else (0.12, 0.1, 0.32, 1)
    bg.inputs["Strength"].default_value = 0.75 if day else 0.4
    sky.inputs["Color"].default_value = (0.5, 0.76, 1.0, 1) if day else (0.13, 0.08, 0.34, 1)
    sky.inputs["Strength"].default_value = 1.0
    nt.links.new(lp.outputs["Is Camera Ray"], mix.inputs["Fac"])
    nt.links.new(bg.outputs["Background"], mix.inputs[1])
    nt.links.new(sky.outputs["Background"], mix.inputs[2])
    nt.links.new(mix.outputs["Shader"], out.inputs["Surface"])


def sun(day=True):
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 10),
                             rotation=(math.radians(42), math.radians(-22), math.radians(25)))
    s = bpy.context.active_object
    s.data.energy = 2.0 if day else 0.3
    s.data.color = (1.0, 0.96, 0.88) if day else (0.6, 0.62, 1.0)
    s.data.angle = math.radians(14)


def camera(target, dist=13.0, pitch=52.0, yaw=0.0, lens=40):
    tx, ty, tz = target
    p = math.radians(pitch)
    cam_loc = (tx + dist * math.cos(p) * math.sin(math.radians(yaw)),
               ty - dist * math.cos(p) * math.cos(math.radians(yaw)),
               tz + dist * math.sin(p))
    bpy.ops.object.camera_add(location=cam_loc)
    cam = bpy.context.active_object
    cam.rotation_euler = (Vector(target) - cam.location).to_track_quat("-Z", "Y").to_euler()
    cam.data.lens = lens
    bpy.context.scene.camera = cam
    return cam


def rolling_world(curvature=0.035, origin_y=0.0, side=0.0):
    """Curve the world around one horizontal axis (the "rolling world")."""
    R = 1.0 / curvature

    def bend(x, y, z):
        t = (y - origin_y) * curvature
        r = R + z
        return (x, origin_y + r * math.sin(t), r * math.cos(t) - R - side * x * x / 2)

    dg = bpy.context.evaluated_depsgraph_get()
    for o in list(bpy.context.scene.objects):
        if o.type != "MESH":
            continue
        ev = o.evaluated_get(dg)
        me = bpy.data.meshes.new_from_object(ev, preserve_all_data_layers=True, depsgraph=dg)
        me.transform(o.matrix_world)
        for v in me.vertices:
            v.co = bend(*v.co)
        me.update()
        o.modifiers.clear()
        o.data = me
        o.matrix_world = Matrix.Identity(4)
    for o in bpy.context.scene.objects:
        if o.type == "LIGHT" and o.data.type == "POINT":
            o.location = bend(*o.location)


# ------------------------------------------------------------------ scenes


def scene_square(night=False):
    build_mats()
    ground()
    plaza(0, 0, 9, 7)
    house(-3.8, 5.6, lit=night)
    house(4.4, 6.4, roof="roof_blue", lit=night)
    fence(-6.6, 3.6, -1.4, 3.6)
    tree(5.8, 1.5, fruit="#f08a4b")
    tree(-6.8, -1.0)
    tree(8.0, -3.5, 0.9)
    tree(-9.0, 6.5, 1.1)
    bush(-1.0, 4.4, 0.8, flower="#ff9cc2")
    dirt(3.2, -3.6, 2.6, 1.8)
    flowers(-2.6, -3.8, "#ff8fb8")
    flowers(1.2, 4.2, "#ffd84d", n=4)
    rock(-5.0, -3.5)
    kid(0.4, -0.6, pattern_mat("kid", "#5aa6e8", "#ffffff", "stripe", cell=28))
    villager(-1.6, 0.6, "cat", pattern_mat("cat", "#f07f95", "#ffe1e8", "dots", cell=40))
    world(not night)
    sun(not night)
    rolling_world(0.03)
    camera((0, 0.6, 0.9), dist=16, pitch=48, lens=38)


def scene_river():
    build_mats()
    ground()
    # Upper terrace: a rounded block with a banded cliff face.
    rbox((0, 10.0, 0.75), (35, 6.5, 0.75), M["cliff"], bev=0.25, seg=4)
    up = bpy.context.active_object
    me = up.data
    uvl = me.uv_layers.active
    for poly in me.polygons:
        for li in poly.loop_indices:
            co = me.vertices[me.loops[li].vertex_index].co
            uvl.data[li].uv = (co.x / 3.0, co.z / 1.5)
    plane(70, 12.6, (0, 10.1, 1.505), M["grass"], subdiv=140)
    for i in range(141):
        sphere((-35 + i * 0.5, 3.55, 1.45), (0.3, 0.14, 0.16), M["fringe"], seg=10, ring=6)
    # River on top, waterfall, pool and river below.
    plane(3, 12, (2.0, 10.0, 1.53), M["water"], subdiv=24)
    plane(3, 1.5, (2.0, 3.47, 0.75), M["fall"], rot=(math.pi / 2, 0, 0), subdiv=6)
    pool = cyl((2.0, 2.0, 0.02), 1.0, 0.04, M["water"], verts=48)
    pool.scale = (3.2, 1.7, 1)
    plane(3.2, 30, (2.0, -11.5, 0.035), M["water"], subdiv=60)
    for i in range(7):
        sphere((0.9 + i * 0.37, 3.2, 0.08), (0.28, 0.2, 0.12), flat("#f4fbff", 0.7), seg=12, ring=6)
    for side_ in (-1, 1):
        for i in range(52):
            rock_y = -26 + i * 0.5
            sphere((2.0 + side_ * 1.65, rock_y, 0.04), (0.22, 0.3, 0.07), flat("#e6d3a8"), seg=10, ring=5)
    # Dock.
    for i in range(4):
        rbox((4.7, -2.0 - i * 0.55, 0.16), (0.75, 0.24, 0.06), M["wood"], bev=0.04)
    for px, py in [(4.05, -1.75), (5.35, -1.75), (4.05, -3.6), (5.35, -3.6)]:
        cyl((px, py, 0.15), 0.12, 0.38, M["bark"], verts=12, bev=0.03)
    tree(-3.5, 1.0, fruit="#f08a4b")
    tree(-1.5, 9.5)
    tree(6.5, 9.0, 1.1)
    tree(7.8, -0.5, 0.9)
    bush(-6.5, -2.5, 1.0, flower="#ffffff")
    flowers(-1.0, -1.8, "#ffffff", n=6)
    rock(0.0, 5.0 + 1.5)
    kid(6.2, -1.6, pattern_mat("kid2", "#7cc36a", "#4f9a46", "check", cell=32))
    world(True)
    sun(True)
    rolling_world(0.03)
    camera((1.5, 0.0, 0.8), dist=15, pitch=46, lens=36)


def scene_lineup():
    build_mats()
    ground(30, 60)
    plaza(1.0, 0.2, 11, 3.2)
    kid(-3.0, 0, pattern_mat("kid", "#5aa6e8", "#ffffff", "stripe", cell=28))
    villager(-1.0, 0, "cat", pattern_mat("cat", "#f07f95", "#ffe1e8", "dots", cell=40))
    villager(1.0, 0, "bear", pattern_mat("bear", "#5f9e5c", "#b7dd7a", "check", cell=32))
    villager(3.0, 0, "frog", pattern_mat("frog", "#f6d860", "#f19a3a", "stripe", cell=28))
    villager(5.0, 0, "bird", pattern_mat("bird", "#ffffff", "#ee6b5b", "check", cell=24))
    world(True)
    sun(True)
    camera((1.0, 0.0, 1.0), dist=12.5, pitch=14, lens=40)


def scene_wide():
    build_mats()
    ground(90, 180)
    plaza(0, 2, 7, 6)
    house(-4, 8.5)
    house(5, 13, roof="roof_blue")
    house(-9, 18, roof="roof_green")
    for i, (x, y, s) in enumerate([(6, 2, 1), (-7, 3, 1.1), (9, 8, 0.9), (-2, 14, 1), (2, 20, 1.2), (-12, 9, 1),
                                   (12, 17, 1.1), (-6, 24, 1), (7, 27, 1), (-14, 22, 1.1), (14, 24, 1), (0, 30, 1.2),
                                   (-9, 32, 1), (10, 33, 1.1)]):
        tree(x, y, s, fruit="#f08a4b" if i % 3 == 0 else None)
    fence(-6.5, 6.6, -1.5, 6.6)
    flowers(2.5, -1.5, "#ff8fb8")
    bush(-3.0, -1.5, 0.8, flower="#ffd84d")
    kid(0, -0.5, pattern_mat("kid3", "#e86a6a", "#ffd84d", "stripe", cell=28))
    world(True)
    sun(True)
    rolling_world(0.06, side=0.035)
    camera((0, 4.0, 0.0), dist=16, pitch=34, lens=28)


def scene_chat():
    build_mats()
    ground(40, 80)
    tree(1.8, 1.8, 1.0, fruit="#f08a4b")
    flowers(-2.0, 0.8, "#ffd84d", n=6)
    dirt(-0.5, -1.6, 2.2, 1.5)
    kid(-0.75, -0.4, pattern_mat("kid4", "#9a7be0", "#ffffff", "dots", cell=40))
    villager(0.75, -0.6, "bear", pattern_mat("bear", "#5f9e5c", "#b7dd7a", "check", cell=32))
    world(True)
    sun(True)
    camera((0, -0.4, 1.15), dist=6.5, pitch=26, lens=45)


def scene_night():
    global NIGHT
    NIGHT = True
    scene_square(night=True)


SCENES = {
    "square": scene_square,
    "night": scene_night,
    "river": scene_river,
    "lineup": scene_lineup,
    "wide": scene_wide,
    "chat": scene_chat,
}


def main():
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    name = argv[0] if argv else "square"
    out = argv[1] if len(argv) > 1 else os.path.join(os.path.dirname(__file__), f"{name}.png")
    w = int(argv[2]) if len(argv) > 2 else 1280
    h = int(argv[3]) if len(argv) > 3 else 960
    sc = reset()
    SCENES[name]()
    sc.render.resolution_x = w
    sc.render.resolution_y = h
    sc.render.filepath = out
    bpy.ops.render.render(write_still=True)
    print("RENDERED", out)


main()
