//! Player sprites. Each pose is a small skeleton (joint positions) that is
//! rasterised once at start-up into a 24x32 bitmap of color *roles*, then
//! given a black outline. At draw time the roles map to a player's palette,
//! which gives every team its own colors from the same art (palette swap).

use crate::gfx::{line_points, Frame};

pub const SW: i32 = 24;
pub const SH: i32 = 32;
/// Origin inside the canvas: between the feet, on the ground.
pub const OX: i32 = 12;
pub const OY: i32 = 30;

// color roles
pub const T: u8 = 0;
pub const OUT: u8 = 1;
pub const SKIN: u8 = 2;
pub const SKIN_D: u8 = 3;
pub const HAIR: u8 = 4;
pub const HAIR_D: u8 = 5;
pub const SHIRT: u8 = 6;
pub const SHIRT_D: u8 = 7;
pub const PANTS: u8 = 8;
pub const PANTS_D: u8 = 9;
pub const WHITE: u8 = 10;
pub const ACC: u8 = 11;
pub const ACC_D: u8 = 12;
pub const SHOE: u8 = 13;
pub const EYE: u8 = 14;
pub const CAP: u8 = 15;
pub const CAP_D: u8 = 16;
pub const ROLES: usize = 17;

pub type Palette = [u32; ROLES];

#[derive(Clone)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

impl Sprite {
    pub fn new(w: i32, h: i32) -> Self {
        Sprite { w, h, px: vec![T; (w * h) as usize] }
    }
    fn set(&mut self, x: i32, y: i32, v: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = v;
        }
    }
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize]
        } else {
            T
        }
    }
    fn thick(&mut self, a: (i32, i32), b: (i32, i32), size: i32, v: u8) {
        let lo = -(size / 2);
        let hi = lo + size;
        line_points(a.0, a.1, b.0, b.1, |x, y| {
            for dy in lo..hi {
                for dx in lo..hi {
                    self.set(x + dx, y + dy, v);
                }
            }
        });
    }
    fn stamp(&mut self, x: i32, y: i32, rows: &[&str], rotate: bool) {
        let h = rows.len() as i32;
        for (ry, row) in rows.iter().enumerate() {
            for (rx, ch) in row.bytes().enumerate() {
                let v = role_of(ch);
                if v == T {
                    continue;
                }
                let (dx, dy) = if rotate {
                    // 90 degrees counter-clockwise: face (right) ends up on top
                    (ry as i32, h - 1 - rx as i32)
                } else {
                    (rx as i32, ry as i32)
                };
                self.set(x + dx, y + dy, v);
            }
        }
    }
    /// Outline a stroke, but only where it lies over the darker back limbs,
    /// so the front arm and leg stay separate from them.
    fn halo(&mut self, a: (i32, i32), b: (i32, i32), size: i32) {
        let lo = -(size / 2);
        let hi = lo + size;
        let mut pts = Vec::new();
        line_points(a.0, a.1, b.0, b.1, |x, y| pts.push((x, y)));
        for (x, y) in pts {
            for dy in lo..hi {
                for dx in lo..hi {
                    if matches!(self.get(x + dx, y + dy), SKIN_D | PANTS_D | ACC_D | SHIRT_D) {
                        self.set(x + dx, y + dy, OUT);
                    }
                }
            }
        }
    }

    /// Add a 1px black outline around the silhouette.
    fn outline(&mut self) {
        let src = self.px.clone();
        let at = |x: i32, y: i32| -> u8 {
            if x >= 0 && y >= 0 && x < self.w && y < self.h {
                src[(y * self.w + x) as usize]
            } else {
                T
            }
        };
        for y in 0..self.h {
            for x in 0..self.w {
                if at(x, y) != T {
                    continue;
                }
                if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                    let v = at(x + dx, y + dy);
                    v != T && v != OUT
                }) {
                    self.px[(y * self.w + x) as usize] = OUT;
                }
            }
        }
    }
}

fn role_of(ch: u8) -> u8 {
    match ch {
        b'O' => OUT,
        b'S' => SKIN,
        b's' => SKIN_D,
        b'H' => HAIR,
        b'h' => HAIR_D,
        b'T' => SHIRT,
        b't' => SHIRT_D,
        b'P' => PANTS,
        b'W' => WHITE,
        b'C' => CAP,
        b'c' => CAP_D,
        b'K' => EYE,
        _ => T,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pose {
    Stand,
    Run1,
    Run2,
    Run3,
    Run4,
    Hold,
    WindUp,
    Throw,
    Catch,
    Jump,
    JumpWind,
    Hit,
    Down,
    Victory,
    Duck,
    Dive,
}

pub const ALL_POSES: [Pose; 16] = [
    Pose::Stand,
    Pose::Run1,
    Pose::Run2,
    Pose::Run3,
    Pose::Run4,
    Pose::Hold,
    Pose::WindUp,
    Pose::Throw,
    Pose::Catch,
    Pose::Jump,
    Pose::JumpWind,
    Pose::Hit,
    Pose::Down,
    Pose::Victory,
    Pose::Duck,
    Pose::Dive,
];

/// Joints, facing right: head, neck, hip, back elbow, back hand, front elbow,
/// front hand, back knee, back foot, front knee, front foot.
type Skel = [(i32, i32); 11];

fn skel(p: Pose) -> Skel {
    match p {
        Pose::Stand => [(12, 5), (12, 10), (12, 19), (8, 15), (8, 20), (16, 15), (16, 20), (10, 24), (10, 29), (14, 24), (14, 29)],
        Pose::Run1 => [(14, 6), (13, 11), (12, 19), (15, 14), (18, 12), (10, 15), (8, 18), (9, 24), (6, 27), (16, 23), (17, 29)],
        Pose::Run2 => [(14, 5), (13, 10), (12, 18), (12, 15), (13, 19), (14, 15), (15, 19), (11, 23), (9, 27), (14, 23), (13, 29)],
        Pose::Run3 => [(14, 6), (13, 11), (12, 19), (10, 15), (8, 18), (16, 14), (19, 12), (16, 23), (17, 29), (9, 24), (6, 27)],
        Pose::Run4 => [(14, 5), (13, 10), (12, 18), (14, 15), (15, 19), (12, 15), (13, 19), (14, 23), (13, 29), (11, 23), (9, 27)],
        Pose::Hold => [(12, 5), (12, 10), (12, 19), (14, 15), (17, 15), (16, 16), (18, 15), (10, 24), (9, 29), (14, 24), (15, 29)],
        Pose::WindUp => [(10, 5), (11, 10), (12, 19), (15, 13), (18, 12), (8, 10), (5, 7), (9, 24), (7, 29), (15, 24), (17, 29)],
        Pose::Throw => [(15, 6), (14, 11), (12, 19), (9, 14), (7, 17), (18, 12), (21, 12), (9, 24), (6, 28), (16, 24), (18, 29)],
        Pose::Catch => [(10, 8), (11, 13), (12, 21), (16, 17), (19, 17), (17, 13), (20, 12), (9, 25), (7, 29), (16, 25), (17, 29)],
        Pose::Jump => [(12, 4), (12, 9), (12, 18), (8, 12), (5, 10), (16, 9), (19, 6), (15, 21), (11, 25), (17, 20), (14, 25)],
        Pose::JumpWind => [(11, 4), (12, 9), (12, 18), (15, 11), (18, 10), (9, 6), (6, 3), (15, 21), (11, 25), (17, 20), (14, 25)],
        Pose::Hit => [(8, 8), (10, 12), (13, 19), (12, 8), (14, 5), (13, 10), (16, 7), (14, 24), (16, 28), (16, 23), (19, 26)],
        Pose::Down => [(5, 25), (9, 27), (17, 27), (10, 23), (12, 21), (12, 30), (15, 30), (20, 24), (23, 27), (21, 27), (23, 29)],
        Pose::Victory => [(12, 5), (12, 10), (12, 19), (8, 7), (4, 2), (16, 7), (20, 2), (10, 24), (9, 29), (14, 24), (15, 29)],
        // crouched, hands over the head
        Pose::Duck => [(13, 15), (12, 20), (10, 25), (15, 15), (16, 11), (17, 16), (17, 11), (15, 25), (12, 29), (16, 26), (16, 29)],
        // flat out, arms first
        Pose::Dive => [(17, 22), (13, 25), (6, 25), (19, 26), (22, 25), (19, 24), (23, 23), (2, 24), (0, 22), (2, 26), (0, 26)],
    }
}

/// Hand position (front hand) in canvas coordinates; the ball sits here.
pub fn hand(p: Pose) -> (i32, i32) {
    let s = skel(p);
    match p {
        Pose::Hold => (19, 14),
        _ => s[6],
    }
}

pub const HAIR_STYLES: usize = 6;

/// Heads, 10x10, facing right.
fn head(style: usize) -> [&'static str; 10] {
    match style {
        // spiky
        0 => [
            "..H.H.H...", ".HHHHHHH..", "HHHHHHHHH.", ".HHHHHHHH.", ".HHhSSSSS.", ".HhsSSSKS.", ".HsSSSSKSS", "..sSSSSSS.",
            "..sSSSSsS.", "...sSSSS..",
        ],
        // long hair with headband
        1 => [
            "...HHHH...", "..HHHHHHH.", ".HHHHHHHHH", ".CCCCCCCCC", "HHHHSSSSS.", "HHhsSSSKS.", "HHsSSSSKSS", "HHhSSSSSS.",
            "HH.sSSSsS.", "H...sSSS..",
        ],
        // cap with brim forward
        2 => [
            "..........", "...CCCCC..", "..CCCCCCC.", ".cCCWCCCC.", ".hcccCCCCC", ".HhsSSSKS.", ".hsSSSSKSS", "..sSSSSSS.",
            "..sSSSSsS.", "...sSSSS..",
        ],
        // beanie with pom-pom
        3 => [
            "....WW....", "...CCCC...", "..CCCCCC..", ".CCCCCCCC.", ".cWcWcWcW.", ".HhsSSSKS.", ".hsSSSSKSS", "..sSSSSSS.",
            "..sSSSSsS.", "...sSSSS..",
        ],
        // mohawk
        4 => [
            "...HHH....", "....HHHH..", "....HHHH..", "..sSHHSS..", ".sSSSSSSS.", ".sSSSSSKS.", ".sSSSSSKSS", "..sSSSSSS.",
            "..sSSSSsS.", "...sSSSS..",
        ],
        // ponytail
        _ => [
            "...HHHH...", "..HHHHHHH.", ".HHHHHHHHH", "HHHHHHHHH.", "HhHhSSSSS.", "Hh.sSSSKS.", "Hh.SSSSKSS", "h..SSSSSS.",
            "...sSSSsS.", "....sSSS..",
        ],
    }
}

fn render(pose: Pose, style: usize) -> Sprite {
    let k = skel(pose);
    let (hd, neck, hip) = (k[0], k[1], k[2]);
    let (bel, bha, fel, fha) = (k[3], k[4], k[5], k[6]);
    let (bkn, bft, fkn, fft) = (k[7], k[8], k[9], k[10]);
    let down = pose == Pose::Down || pose == Pose::Dive;
    let (bsh, fsh) = if down { ((neck.0 + 1, neck.1 - 1), (neck.0 + 1, neck.1 + 1)) } else { ((neck.0 - 1, neck.1 + 2), (neck.0 + 1, neck.1 + 2)) };
    let (bhip, fhip) = if down { ((hip.0, hip.1 - 1), (hip.0, hip.1 + 1)) } else { ((hip.0 - 1, hip.1), (hip.0 + 1, hip.1)) };
    let mid = |a: (i32, i32), b: (i32, i32)| ((a.0 + b.0) / 2, (a.1 + b.1) / 2);
    let mut s = Sprite::new(SW, SH);

    let shoe = |s: &mut Sprite, f: (i32, i32), v: u8| {
        if down {
            s.thick((f.0, f.1 - 2), (f.0, f.1), 2, v);
        } else {
            s.set(f.0, f.1 - 1, WHITE);
            s.set(f.0 + 1, f.1 - 1, WHITE);
            s.thick((f.0 - 1, f.1), (f.0 + 2, f.1), 2, v);
        }
    };

    // back limbs (darker)
    s.thick(bsh, bel, 2, SKIN_D);
    s.thick(bel, bha, 2, SKIN_D);
    s.thick(bsh, mid(bsh, bel), 3, SHIRT_D);
    s.thick(bhip, bkn, 3, PANTS_D);
    s.thick(bkn, bft, 2, SKIN_D);
    shoe(&mut s, bft, ACC_D);
    // torso
    s.thick((neck.0, neck.1 + 1), hip, 7, SHIRT);
    if down {
        s.thick((neck.0 + 1, neck.1 + 2), (hip.0 - 2, hip.1 + 2), 1, SHIRT_D);
    } else {
        // shading down the back of the shirt and a team stripe across the chest
        s.thick((neck.0 - 3, neck.1 + 2), (hip.0 - 3, hip.1 - 2), 1, SHIRT_D);
        s.thick((neck.0 - 2, neck.1 + 5), (neck.0 + 2, neck.1 + 5), 1, ACC);
    }
    s.thick(hip, hip, 7, PANTS);
    // front leg, with a dark edge where it crosses the back limbs
    s.halo(fhip, fkn, 5);
    s.halo(fkn, fft, 4);
    s.thick(fhip, fkn, 3, PANTS);
    s.thick(fkn, fft, 2, SKIN);
    shoe(&mut s, fft, ACC);
    // head
    let rows = head(style);
    s.stamp(hd.0 - 5, hd.1 - 5, &rows, pose == Pose::Down);
    // front arm
    s.halo(fsh, fel, 4);
    s.halo(fel, fha, 4);
    s.thick(fsh, fel, 2, SKIN);
    s.thick(fel, fha, 2, SKIN);
    s.thick(fsh, mid(fsh, fel), 3, SHIRT);
    s.thick(fha, fha, 3, SKIN);
    s.outline();
    s
}

pub struct SpriteBank {
    /// [style][pose]
    pub frames: Vec<Vec<Sprite>>,
}

impl SpriteBank {
    pub fn new() -> Self {
        let frames = (0..HAIR_STYLES).map(|st| ALL_POSES.iter().map(|&p| render(p, st)).collect()).collect();
        SpriteBank { frames }
    }
    pub fn get(&self, style: usize, pose: Pose) -> &Sprite {
        let idx = ALL_POSES.iter().position(|&p| p == pose).unwrap_or(0);
        &self.frames[style % HAIR_STYLES][idx]
    }
}

impl Default for SpriteBank {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw a sprite with its feet at (x, y) on screen.
pub fn blit(f: &mut Frame, s: &Sprite, x: i32, y: i32, flip: bool, pal: &Palette) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v == T {
                continue;
            }
            let px = if flip { x + OX - sx } else { x - OX + sx };
            f.set(px, y - OY + sy, pal[v as usize]);
        }
    }
}

/// Draw only every other pixel (checkerboard): the classic see-through ghost.
pub fn blit_ghost(f: &mut Frame, s: &Sprite, x: i32, y: i32, flip: bool, pal: &Palette, t: u32) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v == T {
                continue;
            }
            let px = if flip { x + OX - sx } else { x - OX + sx };
            let py = y - OY + sy;
            if (px + py + t as i32) & 1 == 0 {
                f.set(px, py, pal[v as usize]);
            }
        }
    }
}

/// Screen position of a sprite-canvas point when the sprite is drawn at (x, y).
pub fn canvas_to_screen(cx: i32, cy: i32, x: i32, y: i32, flip: bool) -> (i32, i32) {
    let px = if flip { x + OX - cx } else { x - OX + cx };
    (px, y - OY + cy)
}
