# ui-from-disc — the game's font and dialogue window from the disc

M4. Sources: m_font.c (+ _main/_rect/_offset/_mark), m_msg_draw_window.c_inc,
m_msg_draw_font.c_inc, m_choice_draw.c_inc. Textures in the REL
(FONT_nes_tex_font1 / _cursor / _choice, window models).

- [x] Font: decode FONT_nes_tex_font1, per-character widths/offsets (mFont_*), draw text in the AC charset as m_font does.
- [x] Dialogue window: the window model/textures and its colours per speaker as m_msg_draw_window does; name plate.
- [x] Choice window (m_choice) for yes/no and menus (trade question uses it; more menus come with the talk topics).
- [x] Without a disc: keep our own open-licence UI.
- [x] Gate passes; live check; Philip compares with retail (his two talk screenshots, 2026-10-13).

Progress (claude): msg_window.rs draws the talk window, name plate and text from the disc on an orthographic overlay camera (game 320x240 x16 space). Open: choice window; window opacity/placement vs a retail screenshot.
