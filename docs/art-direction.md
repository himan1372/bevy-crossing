# Art direction — GameCube-era look

Philip's target: the feel of Animal Crossing on the GameCube, built entirely
from original assets. This note describes that look in our own words, from
the reference pictures Philip collected (kept outside the repo in
`game c\style-refs\`, never committed). Use it for the overall feel —
colour, shape, lighting, proportions. Do not copy specific characters,
buildings, logos or textures.

## Style anchor (Philip, 2026-10-11)
The first blocky, faceted concept pass felt "way off". Philip's anchor is
the CC BY model "Pots the Raccoon" by Lemon Leon (and the same author's
"Macchiato"): **smooth, rounded, soft shapes; a gentle pastel palette;
flat hand-painted colour with a faint brush grain; soft, almost self-lit
shading; expressive painted faces** (eye highlights, eyelids, blush, small
mouth). Where anything below conflicts with this, this wins. Concept
renders: `tools/concept-art/build2.py`.

## Camera
- High, close, slightly tilted-down view (about 45–55° down), following the
  player. Light perspective, not orthographic.
- The ground curves away toward the top of the screen ("rolling world"), so
  the horizon bends and distant things dip out of sight.

## Ground
- The look comes from **painted textures, not geometry**. Low-resolution,
  crisp, hard-edged painting; no blur, no photo detail.
- **Grass:** a carpet of small star/leaf-shaped tufts in two or three
  greens, tightly packed, with darker gaps between them.
- **Worn dirt patches:** tan/yellow ovals with concentric swirl rings.
- **Stone plazas and paths:** chunky blue-grey hexagon/octagon cobbles with
  dark outlines.
- **Cliffs:** stepped terraces. Faces are vertical bands of tan, cream and
  brown; the grass edge overhangs with a jagged, saw-tooth fringe.
- **Water:** saturated deep blue with a white, web-like ripple pattern,
  darker where deep. Rivers have small waterfalls; the sea has scalloped
  wave rows.

## Nature and buildings
- **Trees:** thick trunks with painted bark swirls and knots; canopies built
  from clustered chunky leaf lobes, dark teal-green; fruit as simple round
  balls.
- **Buildings:** small, chunky and boxy, a little toy-like. Bold roof colours
  (reds, oranges), white picket fences, brick or plank walls, warm yellow
  windows at night.

## Characters
- **Big heads:** roughly two heads tall in total; the head is about as big as
  the body.
- Smooth and rounded (no visible facets); the head is wider at the cheeks.
  Faces are painted textures (big eyes with highlights, optional eyelids,
  blush, small nose and mouth), not modelled.
- Short, simple limbs; small hands and feet.
- A soft dark round blob shadow under every character and object.

## Light and colour
- Bright but soft and pastel by day, not loud; texture colour does most of
  the work, with gentle directional shading and a little self-light so
  shadows stay light.
- Night: deep blue-purple sky, warm yellow light from windows and lamps.

## UI
- Rounded, cloud- or bubble-shaped panels with soft pastel patterned
  backgrounds (dots, stripes).
- Chunky, rounded, outlined lettering; round item slots.
- The date and time shown in small cream circles with dark outlines.

## Rules
- Every asset in the repo is original or under a licence that allows it,
  credited in `app/assets/CREDITS.md`.
- No ripped or fan-HD game textures or models, even as tracing sources.
