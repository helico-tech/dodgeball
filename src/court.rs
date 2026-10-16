//! Court backgrounds. The static part is drawn once into a cached frame; the
//! animated bits (crowd, water, snow, lights) are drawn on top every frame.

use crate::game::{proj, Rng, DEPTH, MID, X_MAX, X_MIN};
use crate::gfx::*;
use crate::teams::Court;

struct Fan {
    x: i32,
    y: i32,
    shirt: u32,
    skin: u32,
    hair: u32,
    phase: u32,
    sign: Option<u32>,
}

pub struct Scenery {
    pub court: Court,
    pub bg: Frame,
    fans: Vec<Fan>,
    lights: Vec<(i32, i32, u32)>,
    flakes: Vec<(f32, f32, f32)>,
}

const WHITE_C: u32 = rgb(248, 248, 248);

impl Scenery {
    pub fn new(court: Court, seed: u64) -> Self {
        let mut s = Scenery { court, bg: Frame::new(), fans: Vec::new(), lights: Vec::new(), flakes: Vec::new() };
        let mut rng = Rng(seed | 1);
        match court {
            Court::Beach => s.beach(&mut rng),
            Court::Boardwalk => s.boardwalk(&mut rng),
            Court::Slopes => s.slopes(&mut rng),
            Court::Skatepark => s.skatepark(&mut rng),
        }
        s.make_fans(&mut rng);
        if court == Court::Slopes {
            for _ in 0..60 {
                s.flakes.push((rng.range(0.0, 256.0), rng.range(0.0, 224.0), rng.range(0.3, 1.0)));
            }
        }
        s
    }

    fn make_fans(&mut self, rng: &mut Rng) {
        let (shirts, skins): (Vec<u32>, Vec<u32>) = match self.court {
            Court::Slopes => (
                vec![rgb(232, 48, 48), rgb(48, 112, 232), rgb(248, 200, 32), rgb(56, 184, 88), rgb(232, 88, 168), rgb(248, 136, 32)],
                vec![rgb(248, 200, 160), rgb(224, 160, 104)],
            ),
            Court::Boardwalk => (
                vec![rgb(184, 48, 120), rgb(48, 120, 184), rgb(200, 168, 32), rgb(64, 152, 88), rgb(136, 72, 184)],
                vec![rgb(200, 144, 112), rgb(160, 104, 72), rgb(112, 72, 48)],
            ),
            _ => (
                vec![
                    rgb(232, 48, 48),
                    rgb(48, 160, 232),
                    rgb(248, 216, 48),
                    rgb(64, 200, 96),
                    rgb(248, 104, 184),
                    rgb(248, 248, 248),
                    rgb(152, 80, 224),
                ],
                vec![rgb(248, 200, 160), rgb(224, 160, 104), rgb(160, 96, 56)],
            ),
        };
        let hairs = [rgb(56, 48, 72), rgb(136, 80, 32), rgb(248, 216, 72), rgb(200, 72, 32)];
        // a calmer crowd: 4 shirt colors, dimmer at the back, so players stand out
        let shirts: Vec<u32> = shirts.into_iter().take(4).collect();
        for row in 0..2 {
            let (num, den) = if row == 0 { (3, 4) } else { (7, 8) };
            let y = 103 + row * 5;
            let mut x = 6 + row * 3;
            while x < 250 {
                if rng.f() < 0.85 && !self.blocked(x, row) {
                    let sign = if rng.f() < 0.07 { Some(shirts[rng.int(shirts.len() as i32) as usize]) } else { None };
                    self.fans.push(Fan {
                        x,
                        y,
                        shirt: shade(shirts[rng.int(shirts.len() as i32) as usize], num, den),
                        skin: shade(skins[rng.int(skins.len() as i32) as usize], num, den),
                        hair: hairs[rng.int(4) as usize],
                        phase: rng.next() % 64,
                        sign,
                    });
                }
                x += 6 + rng.int(3);
            }
        }
    }

    fn blocked(&self, x: i32, _row: i32) -> bool {
        match self.court {
            Court::Skatepark => !(28..=228).contains(&x),
            _ => false,
        }
    }

    // ------------------------------------------------------------ courts

    fn beach(&mut self, rng: &mut Rng) {
        let f = &mut self.bg;
        f.band_gradient(
            0,
            86,
            &[
                rgb(40, 24, 88),
                rgb(64, 32, 104),
                rgb(104, 40, 128),
                rgb(152, 56, 136),
                rgb(208, 72, 112),
                rgb(240, 104, 88),
                rgb(248, 152, 80),
                rgb(248, 192, 96),
            ],
        );
        // striped setting sun
        let (cx, cy, r) = (128, 84, 30);
        for y in cy - r..cy {
            let t = (y - (cy - r)) as f32 / r as f32;
            let c = mix(rgb(248, 248, 160), rgb(248, 136, 64), t);
            let gap = y > cy - 16 && (y - (cy - 16)) % 5 < 1 + (y - (cy - 16)) / 6;
            if gap {
                continue;
            }
            let w = (((r * r - (y - cy) * (y - cy)) as f32).sqrt()) as i32;
            f.hline(cx - w, cx + w, y, c);
        }
        // thin cloud streaks
        for (x, y, w) in [(20, 30, 70), (150, 22, 90), (60, 52, 50), (170, 60, 64), (96, 66, 70)] {
            f.hline(x, x + w, y, rgb(184, 72, 128));
            f.hline(x + 6, x + w - 10, y + 1, rgb(144, 56, 120));
        }
        // ocean
        f.band_gradient(86, 100, &[rgb(32, 40, 120), rgb(40, 64, 152), rgb(48, 96, 176)]);
        for y in 86..100 {
            let w = 4 + (y - 86) * 2;
            let mut x = 128 - w;
            while x < 128 + w {
                let len = 2 + rng.int(5);
                if rng.f() < 0.5 {
                    f.hline(x, x + len, y, if y % 2 == 0 { rgb(248, 200, 96) } else { rgb(248, 152, 80) });
                }
                x += len + 2;
            }
        }
        // sand
        f.rect(0, 100, W, H - 100, rgb(240, 200, 120));
        f.dither_rect(0, 100, W, 3, rgb(248, 248, 248), rgb(120, 168, 216));
        f.rect(0, 103, W, 3, rgb(208, 168, 104));
        f.dither_rect(0, 106, W, 2, rgb(208, 168, 104), rgb(240, 200, 120));
        for _ in 0..500 {
            let (x, y) = (rng.int(W), 108 + rng.int(H - 108));
            f.set(x, y, if rng.f() < 0.5 { rgb(216, 176, 96) } else { rgb(248, 224, 160) });
        }
        // palms (silhouettes against the sunset)
        palm(f, 14, 112, 30, 40, 1);
        palm(f, 46, 104, 52, 60, 1);
        palm(f, 242, 112, 226, 36, -1);
        // lifeguard tower
        let tc = rgb(64, 32, 64);
        f.thick_line(196, 104, 200, 74, 2, tc);
        f.thick_line(216, 104, 212, 74, 2, tc);
        f.line(198, 92, 214, 84, tc);
        f.rect(192, 64, 29, 11, rgb(216, 56, 56));
        f.rect(194, 66, 25, 7, rgb(248, 216, 120));
        f.rect(190, 62, 33, 3, rgb(160, 32, 48));
        f.text(195, 66, "LIFE", rgb(160, 32, 48));
        court_floor(f, None, rgb(248, 248, 248), rgb(248, 248, 248));
        // beach umbrellas behind the crowd
        umbrella(f, 82, 101, rgb(232, 48, 48));
        umbrella(f, 168, 101, rgb(48, 160, 232));
        // foreground: a towel and a boombox, very 80s
        f.rect(10, 200, 30, 14, rgb(232, 72, 168));
        for x in (10..40).step_by(6) {
            f.rect(x, 200, 3, 14, rgb(248, 216, 48));
        }
        f.rect(212, 204, 22, 12, rgb(56, 56, 72));
        f.fill_ellipse(217, 210, 3, 3, rgb(160, 160, 176));
        f.fill_ellipse(229, 210, 3, 3, rgb(160, 160, 176));
        f.rect(220, 205, 6, 3, rgb(120, 200, 232));
        f.line(214, 204, 216, 199, rgb(56, 56, 72));
        f.line(232, 204, 230, 199, rgb(56, 56, 72));
        f.hline(216, 230, 199, rgb(56, 56, 72));
    }

    fn boardwalk(&mut self, rng: &mut Rng) {
        let f = &mut self.bg;
        f.band_gradient(
            0,
            96,
            &[rgb(8, 8, 40), rgb(16, 16, 56), rgb(24, 24, 80), rgb(48, 32, 104), rgb(88, 40, 120), rgb(136, 56, 128)],
        );
        for _ in 0..70 {
            let (x, y) = (rng.int(W), rng.int(60));
            f.set(x, y, if rng.f() < 0.3 { rgb(248, 248, 200) } else { rgb(160, 160, 208) });
        }
        f.fill_ellipse(36, 42, 9, 9, rgb(248, 240, 200));
        f.fill_ellipse(39, 40, 7, 7, rgb(24, 24, 80));
        // shops on the left with neon signs
        let bc = rgb(32, 24, 56);
        f.rect(0, 58, 70, 42, bc);
        f.rect(8, 48, 30, 12, bc);
        for wx in (4..66).step_by(10) {
            for wy in (66..92).step_by(9) {
                if rng.f() < 0.6 {
                    f.rect(wx, wy, 5, 5, rgb(248, 200, 96));
                }
            }
        }
        neon(f, 6, 51, "PIZZA", rgb(248, 96, 200), rgb(120, 24, 96));
        neon(f, 44, 60, "SURF", rgb(96, 232, 248), rgb(24, 88, 128));
        // ferris wheel
        let (cx, cy, r) = (190, 56, 30);
        let wc = rgb(152, 136, 200);
        f.thick_line(cx, cy, cx - 18, 100, 2, rgb(88, 72, 136));
        f.thick_line(cx, cy, cx + 18, 100, 2, rgb(88, 72, 136));
        for k in 0..12 {
            let a = k as f32 / 12.0 * std::f32::consts::TAU;
            let (x, y) = (cx + (a.cos() * r as f32) as i32, cy + (a.sin() * r as f32) as i32);
            f.line(cx, cy, x, y, rgb(104, 88, 160));
            self.lights.push((x, y, [rgb(248, 96, 200), rgb(96, 232, 248), rgb(248, 232, 96)][k % 3]));
        }
        f.circle_outline(cx, cy, r, wc);
        f.circle_outline(cx, cy, r - 1, rgb(104, 88, 160));
        f.fill_ellipse(cx, cy, 3, 3, wc);
        // building on the right
        f.rect(228, 52, 28, 48, bc);
        for wy in (58..96).step_by(8) {
            f.rect(234, wy, 4, 4, rgb(248, 200, 96));
            f.rect(244, wy, 4, 4, rgb(96, 232, 248));
        }
        // planks
        f.rect(0, 100, W, H - 100, rgb(152, 96, 56));
        let mut y = 100;
        let mut gap = 3;
        while y < H {
            f.hline(0, W - 1, y, rgb(96, 56, 32));
            let mut x = rng.int(30);
            while x < W {
                f.set(x, y + 1, rgb(96, 56, 32));
                x += 30 + rng.int(40);
            }
            y += gap;
            if gap < 7 && y > 110 + gap * 10 {
                gap += 1;
            }
        }
        for _ in 0..200 {
            f.set(rng.int(W), 100 + rng.int(H - 100), rgb(176, 120, 72));
        }
        // railing behind the crowd
        f.hline(0, W - 1, 97, rgb(200, 200, 216));
        for x in (0..W).step_by(8) {
            f.line(x, 97, x, 100, rgb(160, 160, 184));
        }
        court_floor(f, Some((rgb(136, 80, 48), rgb(152, 96, 56))), rgb(248, 216, 48), rgb(248, 216, 48));
        // lamp posts
        for x in [10, 246] {
            f.thick_line(x, 216, x, 150, 2, rgb(40, 40, 56));
            f.fill_ellipse(x, 148, 4, 3, rgb(248, 248, 200));
            self.lights.push((x, 148, rgb(248, 232, 128)));
        }
    }

    fn slopes(&mut self, rng: &mut Rng) {
        let f = &mut self.bg;
        f.band_gradient(
            0,
            90,
            &[rgb(32, 72, 184), rgb(48, 96, 208), rgb(72, 128, 224), rgb(104, 160, 232), rgb(144, 192, 240), rgb(192, 224, 248)],
        );
        f.fill_ellipse(218, 34, 8, 8, rgb(248, 248, 224));
        for (x, y) in [(30, 30), (110, 22), (160, 44)] {
            f.fill_ellipse(x, y, 14, 5, rgb(248, 248, 248));
            f.fill_ellipse(x + 10, y - 3, 9, 5, rgb(248, 248, 248));
            f.fill_ellipse(x - 8, y + 1, 8, 4, rgb(224, 232, 248));
        }
        // far range
        let far = [(0, 74), (24, 50), (48, 66), (84, 38), (120, 64), (150, 46), (186, 70), (214, 42), (256, 70)];
        mountains(f, &far, rgb(136, 160, 216), rgb(232, 240, 248), 92);
        let near = [(-10, 92), (30, 62), (70, 88), (118, 56), (170, 90), (210, 60), (266, 92)];
        mountains(f, &near, rgb(88, 104, 168), rgb(248, 248, 248), 98);
        // ski lift
        f.line(0, 40, 255, 18, rgb(48, 48, 64));
        for x in [20, 236] {
            let y = 40 - x * 22 / 255;
            f.thick_line(x, y, x, 98, 2, rgb(64, 64, 80));
            f.hline(x - 4, x + 4, y, rgb(64, 64, 80));
        }
        // snow ground
        f.rect(0, 98, W, H - 98, rgb(240, 248, 248));
        f.dither_rect(0, 98, W, 6, rgb(208, 224, 248), rgb(240, 248, 248));
        for _ in 0..300 {
            f.set(rng.int(W), 104 + rng.int(H - 104), rgb(208, 224, 248));
        }
        // pines along the back
        for x in [4, 18, 36, 222, 238, 252] {
            pine(f, x, 104, 18 + rng.int(8));
        }
        court_floor(f, Some((rgb(168, 208, 240), rgb(192, 224, 248))), rgb(216, 40, 56), rgb(48, 88, 216));
        // snowman in the corner
        f.fill_ellipse(236, 206, 9, 8, rgb(248, 248, 248));
        f.fill_ellipse(236, 192, 6, 6, rgb(248, 248, 248));
        f.shade_ellipse(238, 209, 6, 4, 7, 8);
        f.set(234, 190, rgb(24, 24, 40));
        f.set(238, 190, rgb(24, 24, 40));
        f.hline(237, 240, 193, rgb(248, 136, 32));
        f.rect(231, 183, 11, 3, rgb(24, 24, 40));
        f.rect(233, 178, 7, 5, rgb(24, 24, 40));
        f.hline(230, 242, 198, rgb(232, 48, 48));
    }

    fn skatepark(&mut self, rng: &mut Rng) {
        let f = &mut self.bg;
        f.band_gradient(0, 84, &[rgb(56, 136, 224), rgb(80, 160, 232), rgb(112, 184, 240), rgb(152, 208, 248), rgb(200, 232, 248)]);
        for (x, y) in [(46, 24), (190, 34)] {
            f.fill_ellipse(x, y, 16, 5, rgb(248, 248, 248));
            f.fill_ellipse(x + 8, y - 4, 10, 5, rgb(248, 248, 248));
        }
        // skyline
        let mut x = 0;
        while x < W {
            let w = 12 + rng.int(18);
            let h = 18 + rng.int(36);
            f.rect(x, 84 - h, w, h, rgb(120, 136, 176));
            for wy in ((84 - h + 3)..82).step_by(5) {
                for wx in ((x + 2)..(x + w - 2)).step_by(4) {
                    if rng.f() < 0.4 {
                        f.rect(wx, wy, 2, 2, rgb(168, 184, 216));
                    }
                }
            }
            x += w + 1;
        }
        // graffiti wall
        f.rect(0, 80, W, 20, rgb(168, 160, 168));
        f.hline(0, W - 1, 80, rgb(200, 192, 200));
        for x in (0..W).step_by(32) {
            f.line(x, 81, x, 99, rgb(144, 136, 144));
        }
        graffiti(f, 40, 84, "RAD!", rgb(248, 72, 168), rgb(40, 24, 56));
        graffiti(f, 118, 84, "SK8", rgb(128, 248, 64), rgb(24, 56, 24));
        graffiti(f, 184, 84, "GNARLY", rgb(248, 200, 32), rgb(96, 40, 16));
        // chain-link fence over the wall
        for y in 60..80 {
            for x in 0..W {
                if (x + y) % 6 == 0 || (x - y).rem_euclid(6) == 0 {
                    f.set(x, y, rgb(200, 200, 208));
                }
            }
        }
        f.hline(0, W - 1, 60, rgb(136, 136, 152));
        for x in (0..W).step_by(48) {
            f.thick_line(x + 4, 60, x + 4, 80, 2, rgb(136, 136, 152));
        }
        // concrete
        f.rect(0, 100, W, H - 100, rgb(184, 184, 192));
        for _ in 0..600 {
            f.set(rng.int(W), 100 + rng.int(H - 100), if rng.f() < 0.5 { rgb(168, 168, 176) } else { rgb(200, 200, 208) });
        }
        for _ in 0..6 {
            let (mut x, mut y) = (rng.int(W), 104 + rng.int(110));
            for _ in 0..6 {
                let (nx, ny) = (x + rng.int(9) - 4, y + rng.int(5));
                f.line(x, y, nx, ny, rgb(136, 136, 144));
                x = nx;
                y = ny;
            }
        }
        // quarter pipes at the back corners
        quarter_pipe(f, 0, 1);
        quarter_pipe(f, 255, -1);
        court_floor(f, None, rgb(248, 248, 248), rgb(248, 136, 32));
    }

    // ------------------------------------------------------------ ambient

    pub fn draw_ambient(&self, f: &mut Frame, t: u32, excite: f32) {
        match self.court {
            Court::Beach => {
                // wave foam and glints moving on the water
                for k in 0..10 {
                    let y = 87 + (k * 7 % 13);
                    let x = ((k * 53 + t as i32 / (2 + k % 3)) % 300) - 20;
                    f.hline(x, x + 4 + k % 4, y, rgb(144, 184, 232));
                }
                let off = (t / 8) as i32 % 4;
                for x in (0..W).step_by(4) {
                    if (x / 4 + off) % 3 == 0 {
                        f.set(x, 100, rgb(120, 168, 216));
                    }
                }
            }
            Court::Boardwalk => {
                for (i, &(x, y, c)) in self.lights.iter().enumerate() {
                    let on = (t / 12 + i as u32) % 3 != 0;
                    if on {
                        f.fill_ellipse(x, y, 1, 1, c);
                    }
                }
                if (t / 40) % 7 != 0 {
                    neon(f, 6, 51, "PIZZA", rgb(248, 96, 200), rgb(120, 24, 96));
                } else {
                    f.text_bold(6, 51, "PIZZA", rgb(96, 32, 80));
                }
            }
            Court::Slopes => {
                for &(x, y, s) in &self.flakes {
                    let yy = (y + t as f32 * s * 0.6) % 224.0;
                    let xx = (x + (yy * 0.05 + s * 10.0).sin() * 4.0).rem_euclid(256.0);
                    f.set(xx as i32, yy as i32, WHITE_C);
                    if s > 0.8 {
                        f.set(xx as i32 + 1, yy as i32, WHITE_C);
                    }
                }
                // chairs moving up the lift
                for k in 0..5 {
                    let x = ((k * 56 + t as i32 / 3) % 280) - 12;
                    let y = 40 - x * 22 / 255;
                    f.line(x, y, x, y + 6, rgb(48, 48, 64));
                    f.rect(x - 3, y + 6, 7, 2, rgb(216, 40, 56));
                    f.rect(x - 3, y + 3, 1, 3, rgb(216, 40, 56));
                }
            }
            Court::Skatepark => {}
        }
        self.draw_fans(f, t, excite);
        self.draw_boards(f);
    }

    /// Advertising boards along the far side, in front of the crowd.
    fn draw_boards(&self, f: &mut Frame) {
        let words: [&str; 4] = match self.court {
            Court::Beach => ["RAD COLA", "SURF WAX", "HELICO", "TUBULAR"],
            Court::Boardwalk => ["PIZZA", "RAD COLA", "NEON FM", "HELICO"],
            Court::Slopes => ["SKI WAX", "HOT COCOA", "HELICO", "RAD COLA"],
            Court::Skatepark => ["SK8 SHOP", "RAD COLA", "HELICO", "GNARLY"],
        };
        let boards = [
            (rgb(24, 24, 88), rgb(248, 216, 48)),
            (rgb(120, 16, 48), rgb(248, 248, 248)),
            (rgb(16, 88, 64), rgb(248, 248, 248)),
            (rgb(48, 24, 96), rgb(96, 224, 248)),
        ];
        for k in 0..4 {
            let x = k * 64;
            let (bg, fg) = boards[k as usize];
            f.rect(x, 104, 64, 8, bg);
            f.hline(x, x + 63, 104, mix(bg, 0xFFFFFF, 0.4));
            f.hline(x, x + 63, 111, shade(bg, 1, 2));
            f.set(x, 105, shade(bg, 1, 2));
            let w = words[k as usize];
            f.text(x + 32 - text_width(w, false) / 2, 105, w, fg);
        }
    }

    fn draw_fans(&self, f: &mut Frame, t: u32, excite: f32) {
        for fan in &self.fans {
            let jump = if excite > 0.3 && ((t + fan.phase) / 6) % 2 == 0 {
                ((excite * 3.0) as i32).min(3)
            } else if ((t + fan.phase * 5) / 40) % 2 == 0 {
                1
            } else {
                0
            };
            let (x, y) = (fan.x, fan.y - jump);
            let dark = OUTLINE_FAN;
            f.rect(x - 2, y - 5, 5, 5, fan.shirt);
            f.rect(x - 1, y - 8, 3, 3, fan.skin);
            f.hline(x - 1, x + 1, y - 9, fan.hair);
            f.set(x - 3, y - 4, dark);
            f.set(x + 3, y - 4, dark);
            if excite > 0.3 && jump > 0 {
                f.set(x - 3, y - 7, fan.skin);
                f.set(x + 3, y - 7, fan.skin);
                f.set(x - 3, y - 6, fan.shirt);
                f.set(x + 3, y - 6, fan.shirt);
            }
            if let Some(c) = fan.sign {
                f.line(x, y - 9, x, y - 12, rgb(96, 64, 32));
                f.rect(x - 5, y - 18, 11, 6, WHITE_C);
                f.rect(x - 4, y - 17, 9, 4, c);
            }
        }
    }
}

const OUTLINE_FAN: u32 = rgb(40, 32, 56);

/// Court surface and painted lines (trapezoid in fake perspective).
fn court_floor(f: &mut Frame, fill: Option<(u32, u32)>, line: u32, mid: u32) {
    let c = [proj(X_MIN, 0.0, 0.0), proj(X_MAX, 0.0, 0.0), proj(X_MAX, DEPTH, 0.0), proj(X_MIN, DEPTH, 0.0)];
    if let Some((a, b)) = fill {
        for y in c[0].1..=c[2].1 {
            let wy = (y - c[0].1) as f32;
            let (x0, _) = proj(X_MIN, wy, 0.0);
            let (x1, _) = proj(X_MAX, wy, 0.0);
            for x in x0..=x1 {
                f.set(x, y, if (x + y) % 2 == 0 { a } else { b });
            }
        }
    }
    let shadow = |col: u32| shade(col, 3, 4);
    // outer lines, the near line doubled for weight
    f.line(c[0].0, c[0].1, c[1].0, c[1].1, line);
    f.line(c[1].0, c[1].1, c[2].0, c[2].1, line);
    f.line(c[1].0 + 1, c[1].1, c[2].0 + 1, c[2].1, shadow(line));
    f.line(c[3].0, c[3].1, c[0].0, c[0].1, line);
    f.line(c[3].0 - 1, c[3].1, c[0].0 - 1, c[0].1, shadow(line));
    f.hline(c[3].0, c[2].0, c[2].1, line);
    f.hline(c[3].0, c[2].0, c[2].1 + 1, shadow(line));
    // centre line and circle
    let (mx0, my0) = proj(MID, 0.0, 0.0);
    let (mx1, my1) = proj(MID, DEPTH, 0.0);
    f.thick_line(mx0, my0, mx1, my1 + 1, 2, mid);
    let (ccx, ccy) = proj(MID, DEPTH / 2.0, 0.0);
    for k in 0..48 {
        let a = k as f32 / 48.0 * std::f32::consts::TAU;
        f.set(ccx + (a.cos() * 16.0) as i32, ccy + (a.sin() * 7.0) as i32, mid);
    }
    // attack lines
    for x in [MID - 44.0, MID + 44.0] {
        let (ax0, ay0) = proj(x, 0.0, 0.0);
        let (ax1, ay1) = proj(x, DEPTH, 0.0);
        let mut k = 0;
        crate::gfx::line_points(ax0, ay0, ax1, ay1, |x, y| {
            if (k / 3) % 2 == 0 {
                f.set(x, y, line);
            }
            k += 1;
        });
    }
}

fn palm(f: &mut Frame, bx: i32, by: i32, tx: i32, ty: i32, dir: i32) {
    let trunk = rgb(64, 24, 56);
    let leaf = rgb(40, 24, 64);
    let n = 10;
    let mut last = (bx, by);
    for k in 1..=n {
        let t = k as f32 / n as f32;
        let x = bx as f32 + (tx - bx) as f32 * t + (t * (1.0 - t) * 14.0 * dir as f32);
        let y = by as f32 + (ty - by) as f32 * t;
        let p = (x as i32, y as i32);
        f.thick_line(last.0, last.1, p.0, p.1, 4 - (k > 6) as i32, trunk);
        if k % 2 == 0 {
            f.hline(p.0 - 2, p.0 + 1, p.1, rgb(96, 40, 72));
        }
        last = p;
    }
    for (ang, len) in [(-2.8f32, 26.0f32), (-2.2, 24.0), (-1.6, 18.0), (-0.9, 24.0), (-0.3, 26.0), (0.4, 20.0), (2.6, 18.0)] {
        let mut px = tx as f32;
        let mut py = ty as f32;
        let mut a = ang;
        for s in 0..8 {
            let step = len / 8.0;
            let nx = px + a.cos() * step;
            let ny = py + a.sin() * step;
            f.thick_line(px as i32, py as i32, nx as i32, ny as i32, if s < 4 { 3 } else { 2 }, leaf);
            if s > 1 {
                f.set(nx as i32, ny as i32 + 2, leaf);
            }
            px = nx;
            py = ny;
            a += if a.cos() > 0.0 { 0.12 } else { -0.12 };
        }
    }
    f.fill_ellipse(tx, ty + 1, 3, 2, rgb(88, 40, 40));
}

fn umbrella(f: &mut Frame, x: i32, y: i32, c: u32) {
    f.line(x, y - 18, x, y, rgb(96, 64, 48));
    for dy in 0..7 {
        let w = 13 - (6 - dy) * (6 - dy) / 3;
        for dx in -w..=w {
            let stripe = ((dx + 20) / 4) % 2 == 0;
            f.set(x + dx, y - 24 + dy, if stripe { c } else { WHITE_C });
        }
    }
}

fn neon(f: &mut Frame, x: i32, y: i32, s: &str, c: u32, glow: u32) {
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (2, 0), (-2, 0), (0, 2), (0, -2)] {
        f.text_bold(x + dx, y + dy, s, glow);
    }
    f.text_bold(x, y, s, c);
}

fn graffiti(f: &mut Frame, x: i32, y: i32, s: &str, c: u32, o: u32) {
    let grad = [mix(c, 0xFFFFFF, 0.5), c, c, c, shade(c, 3, 4), shade(c, 3, 4), shade(c, 2, 3)];
    f.text_grad(x, y + 2, s, &grad, o, true);
}

fn mountains(f: &mut Frame, pts: &[(i32, i32)], body: u32, snow: u32, base: i32) {
    let mut poly: Vec<(i32, i32)> = pts.to_vec();
    poly.push((pts[pts.len() - 1].0, base));
    poly.push((pts[0].0, base));
    f.fill_poly(&poly, body);
    // snow caps on peaks and a shaded side
    for w in pts.windows(3) {
        let (a, p, b) = (w[0], w[1], w[2]);
        if p.1 < a.1 && p.1 < b.1 {
            let cap = 9;
            let l = (p.0 + (a.0 - p.0) * cap / (a.1 - p.1).max(1), p.1 + cap);
            let r = (p.0 + (b.0 - p.0) * cap / (b.1 - p.1).max(1), p.1 + cap);
            f.fill_poly(&[p, r, (r.0 - 3, r.1 + 2), ((p.0 + r.0) / 2, r.1 - 1), (p.0, r.1 + 3), ((p.0 + l.0) / 2, l.1), l], snow);
            f.fill_poly(&[p, (b.0, b.1), ((b.0 + p.0) / 2, base)], shade(body, 7, 8));
            f.line(p.0, p.1, r.0, r.1, shade(snow, 7, 8));
        }
    }
}

fn pine(f: &mut Frame, x: i32, base: i32, h: i32) {
    f.rect(x - 1, base - 4, 3, 4, rgb(96, 56, 32));
    let green = rgb(24, 96, 72);
    let dark = rgb(16, 64, 48);
    for tier in 0..3 {
        let top = base - h + tier * h / 4;
        let bot = base - 4 - (2 - tier) * h / 6;
        let half = 4 + tier * 2;
        f.fill_poly(&[(x, top), (x + half + 1, bot), (x - half, bot)], green);
        f.fill_poly(&[(x, top), (x + half + 1, bot), (x, bot)], dark);
        f.hline(x - half, x + half, bot, WHITE_C);
        f.set(x, top, WHITE_C);
    }
}

fn quarter_pipe(f: &mut Frame, x: i32, dir: i32) {
    for k in 0..24 {
        let h = 24 - (k * k) / 24;
        let xx = x + k * dir;
        f.line(xx, 100, xx, 100 - h, rgb(160, 160, 176));
        f.set(xx, 100 - h, rgb(232, 232, 240));
    }
    f.thick_line(x, 76, x + 2 * dir, 76, 2, rgb(96, 96, 112));
}
