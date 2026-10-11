"""Concept-art scenes for bevy-crossing (GameCube-era look, all original).

Run:  blender -b --factory-startup --python build.py -- <scene> <out.png> [width height]
Scenes: square, river, lineup, night, wide, chat
"""
import math
import os
import sys

import bpy
import bmesh
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(__file__))
import tex  # noqa: E402

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
        sc.eevee.taa_render_samples = 48
    except AttributeError:
        pass
    return sc


_mat_cache = {}


def mat_image(name, arr, scale=1.0, rough=0.9, alpha=False, emit=0.0, flat_color=None):
    key = (name, scale, alpha, emit)
    if key in _mat_cache:
        return _mat_cache[key]
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    bsdf = nt.nodes.get("Principled BSDF")
    bsdf.inputs["Roughness"].default_value = rough
    for key_name in ("Specular IOR Level", "Specular"):
        if key_name in bsdf.inputs:
            bsdf.inputs[key_name].default_value = 0.15
            break
    if arr is not None:
        img = tex.to_image(name, arr)
        t = nt.nodes.new("ShaderNodeTexImage")
        t.image = img
        t.interpolation = "Closest"
        if scale != 1.0:
            uv = nt.nodes.new("ShaderNodeTexCoord")
            mp = nt.nodes.new("ShaderNodeMapping")
            mp.inputs["Scale"].default_value = (scale, scale, 1)
            nt.links.new(uv.outputs["UV"], mp.inputs["Vector"])
            nt.links.new(mp.outputs["Vector"], t.inputs["Vector"])
        nt.links.new(t.outputs["Color"], bsdf.inputs["Base Color"])
        if alpha:
            nt.links.new(t.outputs["Alpha"], bsdf.inputs["Alpha"])
            m.surface_render_method = "BLENDED" if hasattr(m, "surface_render_method") else None
        if emit > 0:
            nt.links.new(t.outputs["Color"], bsdf.inputs["Emission Color"])
            bsdf.inputs["Emission Strength"].default_value = emit
    else:
        bsdf.inputs["Base Color"].default_value = flat_color
    _mat_cache[key] = m
    return m


def mat_flat(name, color, rough=0.85, emit=0.0):
    key = (name, "flat", emit)
    if key in _mat_cache:
        return _mat_cache[key]
    c = tex.hexrgb(color)
    m = mat_image(name, None, flat_color=tuple(c), rough=rough)
    if emit:
        bsdf = m.node_tree.nodes.get("Principled BSDF")
        bsdf.inputs["Emission Color"].default_value = tuple(c)
        bsdf.inputs["Emission Strength"].default_value = emit
    _mat_cache[key] = m
    return m


def obj_from_mesh(name, verts, faces, mat=None, uvs=None, smooth=False):
    me = bpy.data.meshes.new(name)
    me.from_pydata(verts, [], faces)
    me.update()
    if uvs is not None:
        uvl = me.uv_layers.new()
        for poly in me.polygons:
            for li in poly.loop_indices:
                vi = me.loops[li].vertex_index
                uvl.data[li].uv = uvs[vi]
    o = bpy.data.objects.new(name, me)
    bpy.context.collection.objects.link(o)
    if mat:
        me.materials.append(mat)
    for p in me.polygons:
        p.use_smooth = smooth
    return o


def prim(kind, loc=(0, 0, 0), scale=(1, 1, 1), rot=(0, 0, 0), mat=None, **kw):
    getattr(bpy.ops.mesh, f"primitive_{kind}_add")(location=loc, rotation=rot, **kw)
    o = bpy.context.active_object
    o.scale = scale
    if mat:
        o.data.materials.append(mat)
    return o


def bevel(o, width=0.08, segments=2):
    m = o.modifiers.new("bevel", "BEVEL")
    m.width = width
    m.segments = segments
    m.limit_method = "NONE"
    return o


def plane(name, w, h, loc, mat, rot=(0, 0, 0), subdiv=0):
    if subdiv:
        bpy.ops.mesh.primitive_grid_add(x_subdivisions=subdiv, y_subdivisions=subdiv, size=1, location=loc, rotation=rot)
    else:
        bpy.ops.mesh.primitive_plane_add(size=1, location=loc, rotation=rot)
    o = bpy.context.active_object
    o.name = name
    o.scale = (w, h, 1)
    o.data.materials.append(mat)
    return o


# ------------------------------------------------------------------ props

MATS = {}


def build_mats():
    MATS["grass"] = mat_image("grass", tex.grass(count=900), scale=20)
    MATS["cobble"] = mat_image("cobble", tex.cobbles(), scale=3)
    MATS["dirt"] = mat_image("dirt", tex.dirt_patch(), alpha=True)
    MATS["cliff"] = mat_image("cliff", tex.cliff(), scale=2)
    MATS["water"] = mat_image("water", tex.water(), scale=3, rough=0.3)
    MATS["fall"] = mat_image("fall", tex.waterfall(), rough=0.3)
    MATS["bark"] = mat_image("bark", tex.bark())
    MATS["leaves"] = mat_image("leaves", tex.leaves(), scale=2)
    MATS["roof"] = mat_image("roof", tex.planks(), scale=2)
    MATS["roof_blue"] = mat_image("roof_blue", tex.planks(base="#3d6fc4", line="#264d8f"), scale=2)
    MATS["wall"] = mat_image("wall", tex.wall(), scale=1.5)
    MATS["shadow"] = mat_image("shadow", tex.blob_shadow(), alpha=True)
    MATS["white"] = mat_flat("white", "#f4f1ea")
    MATS["door"] = mat_flat("door", "#7a4a26")
    MATS["wood"] = mat_image("wood", tex.planks(base="#b07a48", line="#7c5230", vertical=True))
    MATS["fringe"] = mat_flat("fringe", "#2f7d32")


def shadow(x, y, r):
    o = plane("shadow", r * 2, r * 2, (x, y, 0.04), MATS["shadow"], subdiv=6)
    return o


def tree(x, y, s=1.0, fruit=None):
    trunk = prim("cone", (x, y, 0.9 * s), (0.32 * s, 0.32 * s, 0.9 * s), mat=MATS["bark"], vertices=8, radius1=1, radius2=0.65, depth=2)
    trunk.data.uv_layers  # cone has UVs by default
    lobes = [(0, 0, 2.3, 1.15), (0.7, 0.2, 2.0, 0.85), (-0.65, -0.1, 2.05, 0.9), (0.1, 0.6, 2.6, 0.8), (-0.2, -0.55, 2.55, 0.8), (0.3, -0.3, 3.0, 0.7)]
    for dx, dy, dz, r in lobes:
        o = prim("ico_sphere", (x + dx * s, y + dy * s, dz * s), (r * s, r * s, r * s * 0.9), mat=MATS["leaves"], subdivisions=1)
    if fruit:
        fm = mat_flat("fruit_" + fruit, fruit)
        for dx, dy, dz in [(0.8, -0.6, 1.9), (-0.7, -0.7, 2.1), (0.2, -1.0, 2.5)]:
            prim("uv_sphere", (x + dx * s, y + dy * s, dz * s), (0.17 * s,) * 3, mat=fm, segments=10, ring_count=6)
    shadow(x, y, 1.3 * s)


def gable_roof(x, y, z, w, d, h, mat):
    hw, hd = w / 2, d / 2
    v = [(-hw, -hd, 0), (hw, -hd, 0), (hw, hd, 0), (-hw, hd, 0), (-hw, 0, h), (hw, 0, h)]
    f = [(0, 1, 5, 4), (2, 3, 4, 5), (0, 4, 3), (1, 2, 5)]
    uvs = [(0, 0), (w / 2, 0), (0, 0), (w / 2, 0), (0, 1), (w / 2, 1)]
    o = obj_from_mesh("roof", [(x + a, y + b, z + c) for a, b, c in v], f, mat, uvs)
    return o


def house(x, y, roof="roof", lit=False, rot=0.0):
    walls = prim("cube", (x, y, 1.0), (1.3, 1.1, 1.0), mat=MATS["wall"])
    bevel(walls, 0.05)
    gable_roof(x, y, 2.0, 3.0, 2.7, 1.4, MATS[roof])
    door = prim("cube", (x, y - 1.11, 0.75), (0.35, 0.03, 0.75), mat=MATS["door"])
    knob = prim("uv_sphere", (x + 0.22, y - 1.16, 0.75), (0.05,) * 3, mat=mat_flat("gold", "#e2b23c"), segments=8, ring_count=5)
    wm = mat_image("win_lit" if lit else "win", tex.window_glow(lit), emit=4.0 if lit else 0.0)
    for wx in (-0.85, 0.85):
        prim("cube", (x + wx, y - 1.11, 1.15), (0.28, 0.03, 0.28), mat=wm)
    prim("cube", (x + 0.8, y + 0.3, 3.0), (0.2, 0.2, 0.45), mat=mat_flat("brick", "#a8503c"))
    prim("cube", (x, y - 1.35, 0.06), (0.55, 0.3, 0.06), mat=mat_flat("step", "#d8d1c2"))
    shadow(x, y, 2.0)
    if lit:
        bpy.ops.object.light_add(type="POINT", location=(x, y - 1.6, 1.3))
        L = bpy.context.active_object
        L.data.energy = 120
        L.data.color = (1.0, 0.8, 0.45)
        L.data.shadow_soft_size = 0.5


def fence(x0, y0, x1, y1, step=0.42):
    length = math.hypot(x1 - x0, y1 - y0)
    n = max(2, int(length / step))
    ang = math.atan2(y1 - y0, x1 - x0)
    for i in range(n + 1):
        t = i / n
        px, py = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
        prim("cube", (px, py, 0.4), (0.07, 0.04, 0.4), rot=(0, 0, ang), mat=MATS["white"])
        prim("cone", (px, py, 0.88), (0.1, 0.06, 0.1), rot=(0, 0, ang + math.pi / 4), mat=MATS["white"], vertices=4, depth=1)
    for z in (0.3, 0.62):
        prim("cube", ((x0 + x1) / 2, (y0 + y1) / 2, z), (length / 2, 0.03, 0.04), rot=(0, 0, ang), mat=MATS["white"])


def flowers(x, y, color, n=5, spread=0.6):
    cm = mat_flat("petal_" + color, color)
    ym = mat_flat("pollen", "#ffd23a")
    sm = mat_flat("stem", "#2e7a2e")
    for i in range(n):
        a = i * 2.39996
        r = spread * math.sqrt((i + 0.5) / n)
        fx, fy = x + r * math.cos(a), y + r * math.sin(a)
        prim("cylinder", (fx, fy, 0.15), (0.025, 0.025, 0.15), mat=sm, vertices=6)
        for k in range(5):
            pa = k * 2 * math.pi / 5
            prim("uv_sphere", (fx + 0.09 * math.cos(pa), fy + 0.09 * math.sin(pa), 0.32), (0.07, 0.07, 0.035), mat=cm, segments=8, ring_count=4)
        prim("uv_sphere", (fx, fy, 0.33), (0.05, 0.05, 0.03), mat=ym, segments=8, ring_count=4)


# ------------------------------------------------------------------ characters


def face_plate(x, y, z, w, h, face_arr, name, facing=0.0):
    m = mat_image(name, face_arr, alpha=True, rough=1.0)
    o = plane(name, w, h, (0, 0, 0), m, rot=(math.pi / 2, 0, 0))
    o.location = (x, y, z)
    o.rotation_euler = (math.pi / 2, 0, facing)
    return o


def body_parts(x, y, shirt, legs="#3a5aa0", shoes="#c0392b", skin="#f2c79a", facing=0.0, arms_out=0.35):
    sk = mat_flat("skin_" + skin, skin)
    prim("cone", (x, y, 0.62), (0.34, 0.3, 0.36), mat=shirt, vertices=10, radius1=1, radius2=0.72, depth=1)
    for side in (-1, 1):
        prim("cylinder", (x + side * 0.12, y, 0.2), (0.07, 0.07, 0.17), mat=mat_flat("legs_" + legs, legs), vertices=8)
        prim("cube", (x + side * 0.12, y - 0.05, 0.05), (0.09, 0.13, 0.05), mat=mat_flat("shoe_" + shoes, shoes))
        arm = prim("cylinder", (x + side * 0.42, y, 0.72), (0.06, 0.06, 0.2), rot=(0, side * (math.pi / 2 - arms_out), 0), mat=shirt, vertices=8)
        prim("uv_sphere", (x + side * 0.6, y, 0.62), (0.08,) * 3, mat=sk, segments=8, ring_count=6)


def kid(x, y, shirt_arr, hat="#d94b4b", hair="#5a3a22", skin="#f2c79a", eyes="#2a2a2a"):
    """Original player-kid: big boxy head, beanie with pom-pom, painted face."""
    shirt = mat_image("shirt_kid_" + hat, shirt_arr, scale=1.0)
    body_parts(x, y, shirt, skin=skin)
    head = prim("cube", (x, y, 1.42), (0.48, 0.44, 0.46), mat=mat_flat("skin_" + skin, skin))
    bevel(head, 0.62, 4)
    face_plate(x, y - 0.452, 1.36, 0.82, 0.82, tex.face(eye=eyes), "face_kid_" + hat)
    beanie = prim("uv_sphere", (x, y + 0.02, 1.78), (0.52, 0.49, 0.32), mat=mat_flat("hat_" + hat, hat), segments=16, ring_count=8)
    prim("cylinder", (x, y + 0.02, 1.66), (0.53, 0.5, 0.07), mat=mat_flat("hatband", "#f4f1ea"), vertices=16)
    prim("uv_sphere", (x, y + 0.02, 2.12), (0.13,) * 3, mat=mat_flat("pom", "#f4f1ea"), segments=10, ring_count=6)
    for side in (-1, 1):
        prim("cube", (x + side * 0.47, y + 0.08, 1.32), (0.07, 0.3, 0.26), mat=mat_flat("hair_" + hair, hair))
    shadow(x, y, 0.6)


def villager(x, y, kind, fur, shirt_arr, accent="#ffffff", facing=0.0):
    """Original animal villagers: cat, bear, frog, bird."""
    shirt = mat_image(f"shirt_{kind}", shirt_arr)
    body_parts(x, y, shirt, legs=fur, shoes=fur, skin=fur)
    fm = mat_flat(f"fur_{kind}", fur)
    head = prim("cube", (x, y, 1.4), (0.5, 0.45, 0.44), mat=fm)
    bevel(head, 0.62, 4)
    nose = "#e8743a"
    if kind == "cat":
        for side in (-1, 1):
            prim("cone", (x + side * 0.3, y, 1.95), (0.17, 0.12, 0.2), mat=fm, vertices=3, depth=1)
        prim("cube", (x, y - 0.44, 1.26), (0.2, 0.05, 0.12), mat=mat_flat("muzzle_" + accent, accent))
        nose = "#e86b8a"
    elif kind == "bear":
        for side in (-1, 1):
            prim("uv_sphere", (x + side * 0.38, y, 1.85), (0.17, 0.1, 0.17), mat=fm, segments=12, ring_count=6)
        prim("uv_sphere", (x, y - 0.45, 1.25), (0.2, 0.1, 0.14), mat=mat_flat("muzzle_" + accent, accent), segments=12, ring_count=6)
        nose = "#3a2a20"
    elif kind == "frog":
        for side in (-1, 1):
            prim("uv_sphere", (x + side * 0.24, y - 0.05, 1.9), (0.17,) * 3, mat=fm, segments=12, ring_count=6)
        nose = None
    elif kind == "bird":
        prim("cone", (x, y - 0.6, 1.3), (0.15, 0.15, 0.22), rot=(math.pi / 2, 0, 0), mat=mat_flat("beak", "#f0a21a"), vertices=8, depth=1)
        prim("cone", (x, y + 0.05, 1.98), (0.06, 0.18, 0.18), mat=fm, vertices=3, depth=1)
        nose = None
    eyes_up = 0.12 if kind == "frog" else 0.0
    face_plate(x, y - 0.452, 1.38, 0.86, 0.82, tex.face(nose=nose, eyes_up=eyes_up, blush=kind != "frog"), f"face_{kind}")
    shadow(x, y, 0.62)


# ------------------------------------------------------------------ world, light, camera


def ground(size=70, subdiv=140):
    g = plane("ground", size, size, (0, 0, 0), MATS["grass"], subdiv=subdiv)
    return g


def dirt(x, y, w, h):
    plane("dirt", w, h, (x, y, 0.03), MATS["dirt"], subdiv=10)


def plaza(x, y, w, h):
    p = plane("plaza", w, h, (x, y, 0.025), MATS["cobble"], subdiv=40)
    return p


def world(day=True):
    """Sky colour for the camera, softer light for the scene."""
    w = bpy.data.worlds.new("world")
    bpy.context.scene.world = w
    w.use_nodes = True
    nt = w.node_tree
    out = nt.nodes.get("World Output")
    bg = nt.nodes.get("Background")
    sky = nt.nodes.new("ShaderNodeBackground")
    mix = nt.nodes.new("ShaderNodeMixShader")
    lp = nt.nodes.new("ShaderNodeLightPath")
    bg.inputs["Color"].default_value = (0.55, 0.78, 1.0, 1) if day else (0.12, 0.08, 0.3, 1)
    bg.inputs["Strength"].default_value = 0.55 if day else 0.35
    sky.inputs["Color"].default_value = (0.42, 0.72, 1.0, 1) if day else (0.16, 0.09, 0.38, 1)
    sky.inputs["Strength"].default_value = 1.0
    nt.links.new(lp.outputs["Is Camera Ray"], mix.inputs["Fac"])
    nt.links.new(bg.outputs["Background"], mix.inputs[1])
    nt.links.new(sky.outputs["Background"], mix.inputs[2])
    nt.links.new(mix.outputs["Shader"], out.inputs["Surface"])


def sun(day=True):
    bpy.ops.object.light_add(type="SUN", location=(0, 0, 10), rotation=(math.radians(40), math.radians(-25), math.radians(20)))
    s = bpy.context.active_object
    s.data.energy = 2.3 if day else 0.35
    s.data.color = (1.0, 0.97, 0.9) if day else (0.55, 0.6, 1.0)
    s.data.angle = math.radians(8)


def camera(target, dist=13.0, pitch=52.0, yaw=0.0, lens=40):
    tx, ty, tz = target
    p = math.radians(pitch)
    cx = tx + dist * math.cos(p) * math.sin(math.radians(yaw))
    cy = ty - dist * math.cos(p) * math.cos(math.radians(yaw))
    cz = tz + dist * math.sin(p)
    bpy.ops.object.camera_add(location=(cx, cy, cz))
    cam = bpy.context.active_object
    direction = Vector(target) - cam.location
    cam.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
    cam.data.lens = lens
    bpy.context.scene.camera = cam
    return cam


def rolling_world(curvature=0.035, origin_y=0.0, side=0.0):
    """Curve the whole world around one horizontal axis so the ground falls
    away from the camera (the "rolling world" horizon).

    Every mesh is baked to world space (modifiers applied) and bent with the
    same formula: a point at distance d along +y from `origin_y` moves onto a
    circle of radius R = 1/curvature (plus its height)."""
    R = 1.0 / curvature
    dg = bpy.context.evaluated_depsgraph_get()
    for o in list(bpy.context.scene.objects):
        if o.type != "MESH":
            continue
        ev = o.evaluated_get(dg)
        me = bpy.data.meshes.new_from_object(ev, preserve_all_data_layers=True, depsgraph=dg)
        me.transform(o.matrix_world)
        for v in me.vertices:
            x, y, z = v.co
            t = (y - origin_y) * curvature
            r = R + z
            v.co = (x, origin_y + r * math.sin(t), r * math.cos(t) - R - side * x * x / 2)
        me.update()
        o.modifiers.clear()
        o.data = me
        o.matrix_world = Matrix.Identity(4)
    # Lights follow the same bend so lamps stay by their houses.
    for o in bpy.context.scene.objects:
        if o.type == "LIGHT" and o.data.type == "POINT":
            x, y, z = o.location
            t = (y - origin_y) * curvature
            r = R + z
            o.location = (x, origin_y + r * math.sin(t), r * math.cos(t) - R - side * x * x / 2)


# ------------------------------------------------------------------ scenes


def scene_square(night=False):
    build_mats()
    ground()
    plaza(0, 0, 9, 7)
    house(-3.6, 5.2, lit=night)
    house(4.2, 6.0, roof="roof_blue", lit=night)
    fence(-6.2, 3.4, -1.2, 3.4)
    tree(5.6, 1.5, fruit="#e0452c")
    tree(-6.5, -1.0)
    tree(7.8, -3.5, 0.9)
    tree(-8.5, 6.5, 1.1)
    dirt(3.2, -3.5, 2.6, 1.8)
    flowers(-2.4, -3.8, "#ff6fa5")
    flowers(-1.4, 3.9, "#ffd23a", n=4)
    kid(0.4, -0.6, tex.pattern("#2f8fd8", "#ffffff", "stripe", cell=12))
    villager(-1.6, 0.6, "cat", "#f0b55a", tex.pattern("#e85d75", "#ffd1dc", "dots", cell=18))
    world(not night)
    sun(not night)
    rolling_world(0.03, origin_y=0.0)
    camera((0, -0.2, 0.9), dist=14, pitch=48, lens=38)


def scene_river():
    build_mats()
    # Lower ground and an upper terrace behind a cliff.
    ground()
    upper = plane("upper", 70, 12, (0, 9.5, 1.5), MATS["grass"], subdiv=140)
    cliff_face = plane("cliff_face", 70, 1.5, (0, 3.49, 0.75), MATS["cliff"], rot=(math.pi / 2, 0, 0))
    # Saw-tooth grass fringe hanging over the cliff edge.
    verts, faces = [], []
    for i in range(140):
        x0 = -35 + i * 0.5
        a = len(verts)
        verts += [(x0, 3.45, 1.5), (x0 + 0.5, 3.45, 1.5), (x0 + 0.25, 3.42, 1.5 - 0.35)]
        faces.append((a, a + 1, a + 2))
    obj_from_mesh("fringe", verts, faces, MATS["fringe"])
    # River: flows across the upper terrace, falls, then runs toward camera.
    plane("river_up", 3, 12, (2.0, 9.5, 1.53), MATS["water"], subdiv=24)
    plane("fall", 3, 1.5, (2.0, 3.46, 0.75), MATS["fall"], rot=(math.pi / 2, 0, 0))
    plane("river_low", 3.2, 30, (2.0, -11.5, 0.035), MATS["water"], subdiv=60)
    plane("pool", 6, 3, (2.0, 2.0, 0.036), MATS["water"], subdiv=12)
    for side in (-1, 1):
        plane("bank", 0.25, 30, (2.0 + side * 1.7, -11.5, 0.045), MATS["cliff"], subdiv=60)
    # Dock.
    for i in range(4):
        prim("cube", (4.6, -2.0 - i * 0.55, 0.14), (0.7, 0.24, 0.06), mat=MATS["wood"])
    for px, py in [(4.0, -1.7), (5.2, -1.7), (4.0, -3.6), (5.2, -3.6)]:
        prim("cylinder", (px, py, 0.12), (0.12, 0.12, 0.25), mat=MATS["bark"], vertices=10)
    tree(-3.5, 1.0, fruit="#e0452c")
    tree(-1.5, 9.0)
    tree(6.5, 8.5, 1.1)
    tree(7.5, -0.5, 0.9)
    flowers(-1.0, -1.8, "#ffffff", n=6)
    kid(6.0, -1.4, tex.pattern("#5cbf48", "#2f7d32", "check", cell=16))
    world(True)
    sun(True)
    rolling_world(0.03)
    camera((1.5, 0.0, 0.8), dist=15, pitch=46, lens=36)


def scene_lineup():
    build_mats()
    ground(30, 60)
    plaza(0, 0.2, 9, 3.2)
    kid(-3.0, 0, tex.pattern("#2f8fd8", "#ffffff", "stripe", cell=12))
    villager(-1.0, 0, "cat", "#f0b55a", tex.pattern("#e85d75", "#ffd1dc", "dots", cell=18))
    villager(1.0, 0, "bear", "#9c6a43", tex.pattern("#3d7a3a", "#a8d45a", "check", cell=16), accent="#e8cfa0")
    villager(3.0, 0, "frog", "#6cc04a", tex.pattern("#f2d14a", "#e88a1a", "stripe", cell=14))
    villager(5.0, 0.0, "bird", "#7fb2e6", tex.pattern("#ffffff", "#e0452c", "check", cell=12))
    world(True)
    sun(True)
    camera((1.0, 0.0, 1.0), dist=12.5, pitch=16, lens=40)


def scene_wide():
    build_mats()
    ground(90, 180)
    plaza(0, 2, 7, 6)
    house(-4, 8)
    house(5, 13, roof="roof_blue")
    house(-9, 18)
    for i, (x, y, s) in enumerate([(6, 2, 1), (-7, 3, 1.1), (9, 8, 0.9), (-2, 14, 1), (2, 20, 1.2), (-12, 9, 1), (12, 17, 1.1), (-6, 24, 1), (7, 27, 1), (-14, 22, 1.1), (14, 24, 1), (0, 30, 1.2), (-9, 32, 1), (10, 33, 1.1)]):
        tree(x, y, s, fruit="#e0452c" if i % 3 == 0 else None)
    fence(-6.5, 6.2, -1.5, 6.2)
    flowers(2.5, -1.5, "#ff6fa5")
    kid(0, -0.5, tex.pattern("#d94b4b", "#ffd23a", "stripe", cell=12))
    world(True)
    sun(True)
    rolling_world(0.06, origin_y=0.0, side=0.035)
    camera((0, 4.0, 0.0), dist=16, pitch=34, lens=28)


def scene_chat():
    build_mats()
    ground(40, 80)
    tree(1.6, 1.6, 1.0, fruit="#e0452c")
    flowers(-2.0, 0.8, "#ffd23a", n=6)
    dirt(-0.5, -1.5, 2.2, 1.5)
    kid(-0.7, -0.4, tex.pattern("#7f5ad6", "#ffffff", "dots", cell=16))
    villager(0.7, -0.6, "bear", "#9c6a43", tex.pattern("#3d7a3a", "#a8d45a", "check", cell=16), accent="#e8cfa0")
    world(True)
    sun(True)
    camera((0, -0.4, 1.1), dist=6.5, pitch=32, lens=45)


SCENES = {
    "square": scene_square,
    "night": lambda: scene_square(night=True),
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
