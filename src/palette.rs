use egui::Color32;

const PALETTE: [(u8, u8, u8); 32] = [
    (230,  25,  75),  // red
    ( 60, 180,  75),  // green
    (255, 225,  25),  // yellow
    (  0, 130, 200),  // blue
    (245, 130,  48),  // orange
    (145,  30, 180),  // purple
    ( 70, 240, 240),  // cyan
    (240,  50, 230),  // magenta
    (210, 245,  60),  // lime
    (250, 190, 212),  // pink
    (  0, 128, 128),  // teal
    (220, 190, 255),  // lavender
    (170, 110,  40),  // brown
    (255, 250, 200),  // beige
    (128,   0,   0),  // maroon
    (170, 255, 195),  // mint
    (128, 128,   0),  // olive
    (255, 215, 180),  // apricot
    (  0,   0, 128),  // navy
    (128, 128, 128),  // grey
    (255,  80,  80),  // coral
    ( 80, 200, 120),  // emerald
    (255, 180,   0),  // amber
    ( 80, 120, 255),  // periwinkle
    (200,  80,   0),  // rust
    (180,   0, 180),  // violet
    (  0, 200, 200),  // sky
    (200,   0,  80),  // crimson
    (160, 220,  80),  // yellow-green
    (  0, 160, 100),  // jade
    (220, 130, 220),  // mauve
    (100,  60,   0),  // chocolate
];

pub fn node_color(i: usize, _n: usize) -> Color32 {
    let (r, g, b) = PALETTE[i % PALETTE.len()];
    Color32::from_rgb(r, g, b)
}
