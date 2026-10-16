//! Scenes and flow: title, team select, versus card, match, continue,
//! game over and the champion screen. Pure logic + drawing into a Frame;
//! the window, keyboard and speakers live in main.rs.

use crate::audio::Track;
use crate::court::Scenery;
use crate::game::*;
use crate::gfx::*;
use crate::render::*;
use crate::sprites::{blit, Palette, Pose, Sprite, SpriteBank, OX, OY, T};
use crate::teams::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    One,
    Two,
    Demo,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    Title,
    Select,
    Versus,
    Play,
    Continue,
    GameOver,
    Champion,
}

pub const CPU_SKILL: [f32; 3] = [0.5, 0.68, 0.84];

pub struct App {
    pub scene: Scene,
    pub t: u32,
    pub mode: Mode,
    pub menu: usize,
    pub menu_open: bool,
    pub cursor: [usize; 2],
    pub locked: [bool; 2],
    pub ladder: Vec<usize>,
    pub stage: usize,
    pub game: Option<Match>,
    pub scenery: Option<Scenery>,
    pub bank: SpriteBank,
    title_bg: Scenery,
    pub sfx: Vec<Sfx>,
    prev: [Pad; 2],
    rng: Rng,
    pub quit: bool,
    pub paused: bool,
    idle: u32,
    pub count: i32,
    pub frame: u32,
    confetti: Vec<(f32, f32, f32, u32)>,
}

fn edge(now: Pad, prev: Pad) -> Pad {
    Pad {
        up: now.up && !prev.up,
        down: now.down && !prev.down,
        left: now.left && !prev.left,
        right: now.right && !prev.right,
        a: now.a && !prev.a,
        b: now.b && !prev.b,
        c: now.c && !prev.c,
        start: now.start && !prev.start,
    }
}

fn any(p: Pad) -> bool {
    p.a || p.b || p.c || p.start || p.up || p.down || p.left || p.right
}

impl App {
    pub fn new(seed: u64) -> Self {
        App {
            scene: Scene::Title,
            t: 0,
            mode: Mode::One,
            menu: 0,
            menu_open: false,
            cursor: [0, 1],
            locked: [false; 2],
            ladder: Vec::new(),
            stage: 0,
            game: None,
            scenery: None,
            bank: SpriteBank::new(),
            title_bg: Scenery::new(Court::Beach, 7),
            sfx: Vec::new(),
            prev: [Pad::default(); 2],
            rng: Rng(seed | 1),
            quit: false,
            paused: false,
            idle: 0,
            count: 0,
            frame: 0,
            confetti: Vec::new(),
        }
    }

    pub fn music(&self) -> Option<Track> {
        match self.scene {
            Scene::Title | Scene::Select | Scene::Versus => Some(Track::Title),
            Scene::Play => Some(Track::Match),
            Scene::Champion => Some(Track::Champion),
            Scene::Continue | Scene::GameOver => None,
        }
    }

    fn go(&mut self, s: Scene) {
        self.scene = s;
        self.t = 0;
    }

    /// Start a match between two crews. Side 0 is player 1 in 1P and 2P.
    pub fn start_match(&mut self, teams: [usize; 2], ctrls: [Ctrl; 2], skills: [f32; 2], court: Court) {
        let seed = self.rng.next() as u64;
        self.game = Some(Match::new(teams, ctrls, skills, court, seed));
        self.scenery = Some(Scenery::new(court, seed ^ 0x55));
        self.paused = false;
        self.go(Scene::Play);
    }

    pub fn start_demo(&mut self) {
        let a = self.rng.int(4) as usize;
        let b = (a + 1 + self.rng.int(3) as usize) % 4;
        self.mode = Mode::Demo;
        self.start_match([a, b], [Ctrl::Cpu, Ctrl::Cpu], [0.6, 0.6], TEAMS[b].court);
    }

    fn start_stage(&mut self) {
        let opp = self.ladder[self.stage];
        // your CPU teammates dodge as well as the crew you face (at least 0.55)
        let mate = CPU_SKILL[self.stage].max(0.55);
        self.start_match([self.cursor[0], opp], [Ctrl::Human(0), Ctrl::Cpu], [mate, CPU_SKILL[self.stage]], TEAMS[opp].court);
    }

    pub fn update(&mut self, pads: [Pad; 2], esc: bool) {
        self.frame = self.frame.wrapping_add(1);
        self.t += 1;
        let e = [edge(pads[0], self.prev[0]), edge(pads[1], self.prev[1])];
        self.prev = pads;
        let ok = |p: Pad| p.a || p.start;
        if any(e[0]) || any(e[1]) {
            self.idle = 0;
        } else {
            self.idle += 1;
        }

        if esc {
            match self.scene {
                Scene::Title => self.quit = true,
                _ => {
                    self.game = None;
                    self.menu_open = false;
                    self.go(Scene::Title);
                }
            }
            return;
        }

        match self.scene {
            Scene::Title => {
                if !self.menu_open {
                    if ok(e[0]) && self.t > 20 {
                        self.menu_open = true;
                        self.sfx.push(Sfx::Confirm);
                    }
                    if self.idle > 1200 {
                        self.start_demo();
                    }
                } else {
                    if e[0].up || e[0].down {
                        self.menu = 1 - self.menu;
                        self.sfx.push(Sfx::Blip);
                    }
                    if e[0].b {
                        self.menu_open = false;
                    }
                    if ok(e[0]) {
                        self.mode = if self.menu == 0 { Mode::One } else { Mode::Two };
                        self.locked = [false; 2];
                        self.cursor = [0, 1];
                        self.sfx.push(Sfx::Confirm);
                        self.go(Scene::Select);
                    }
                    if self.idle > 1200 {
                        self.menu_open = false;
                        self.start_demo();
                    }
                }
            }
            Scene::Select => {
                let players = if self.mode == Mode::Two { 2 } else { 1 };
                for pi in 0..players {
                    let p = e[pi];
                    if !self.locked[pi] {
                        let c = self.cursor[pi];
                        let n = match (p.left, p.right, p.up, p.down) {
                            (true, ..) | (_, true, ..) => Some(c ^ 1),
                            (_, _, true, _) | (_, _, _, true) => Some(c ^ 2),
                            _ => None,
                        };
                        if let Some(n) = n {
                            self.cursor[pi] = n;
                            self.sfx.push(Sfx::Blip);
                        }
                        if ok(p) {
                            let other = 1 - pi;
                            if players == 2 && self.locked[other] && self.cursor[other] == self.cursor[pi] {
                                self.sfx.push(Sfx::Bounce);
                            } else {
                                self.locked[pi] = true;
                                self.sfx.push(Sfx::Confirm);
                            }
                        }
                        if p.b && pi == 0 {
                            self.go(Scene::Title);
                            self.menu_open = true;
                            return;
                        }
                    } else if p.b {
                        self.locked[pi] = false;
                    }
                }
                if self.locked[..players].iter().all(|&l| l) && self.t > 10 {
                    if self.mode == Mode::One {
                        let me = self.cursor[0];
                        // opponents in order of the crews after yours
                        self.ladder = (1..4).map(|k| (me + k) % 4).collect();
                        self.stage = 0;
                    }
                    self.go(Scene::Versus);
                }
            }
            Scene::Versus => {
                if self.t > 150 || (self.t > 30 && (ok(e[0]) || ok(e[1]))) {
                    match self.mode {
                        Mode::One => self.start_stage(),
                        _ => {
                            let (a, b) = (self.cursor[0], self.cursor[1]);
                            self.start_match([a, b], [Ctrl::Human(0), Ctrl::Human(1)], [0.5, 0.5], TEAMS[b].court);
                        }
                    }
                }
            }
            Scene::Play => {
                if self.mode == Mode::Demo {
                    if any(e[0]) || any(e[1]) {
                        self.game = None;
                        self.go(Scene::Title);
                        return;
                    }
                } else if e[0].start || e[1].start {
                    self.paused = !self.paused;
                    self.sfx.push(Sfx::Blip);
                }
                if self.paused {
                    return;
                }
                let Some(g) = self.game.as_mut() else {
                    self.go(Scene::Title);
                    return;
                };
                g.update(&pads);
                self.sfx.append(&mut g.sfx);
                if g.finished() && g.phase_t > 200 {
                    let winner = g.winner;
                    match self.mode {
                        Mode::One => {
                            if winner == Some(0) {
                                self.stage += 1;
                                if self.stage >= self.ladder.len() {
                                    self.make_confetti();
                                    self.go(Scene::Champion);
                                } else {
                                    self.go(Scene::Versus);
                                }
                            } else {
                                self.count = 9;
                                self.go(Scene::Continue);
                            }
                        }
                        _ => {
                            self.menu_open = false;
                            self.go(Scene::Title);
                        }
                    }
                }
            }
            Scene::Continue => {
                if self.t % 60 == 0 {
                    self.count -= 1;
                    self.sfx.push(Sfx::Blip);
                }
                if ok(e[0]) && self.t > 20 {
                    self.sfx.push(Sfx::Confirm);
                    self.go(Scene::Versus);
                } else if self.count < 0 {
                    self.go(Scene::GameOver);
                }
            }
            Scene::GameOver => {
                if self.t > 240 || (self.t > 60 && ok(e[0])) {
                    self.menu_open = false;
                    self.go(Scene::Title);
                }
            }
            Scene::Champion => {
                if self.t % 90 == 1 {
                    self.sfx.push(Sfx::Cheer);
                }
                for c in self.confetti.iter_mut() {
                    c.1 += c.2;
                    c.0 += ((c.1 * 0.05) + c.2 * 9.0).sin() * 0.4;
                    if c.1 > 224.0 {
                        c.1 -= 230.0;
                    }
                }
                if self.t > 900 || (self.t > 120 && ok(e[0])) {
                    self.menu_open = false;
                    self.go(Scene::Title);
                }
            }
        }
    }

    fn make_confetti(&mut self) {
        self.confetti.clear();
        let cols = [rgb(248, 56, 72), rgb(248, 216, 48), rgb(64, 200, 248), rgb(120, 248, 96), rgb(248, 104, 200)];
        for _ in 0..90 {
            let c = cols[self.rng.int(5) as usize];
            self.confetti.push((self.rng.range(0.0, 256.0), self.rng.range(-224.0, 224.0), self.rng.range(0.4, 1.2), c));
        }
    }

    // ---------------------------------------------------------------- draw

    pub fn draw(&self, f: &mut Frame) {
        let t = self.frame;
        match self.scene {
            Scene::Title => self.draw_title(f, t),
            Scene::Select => self.draw_select(f, t),
            Scene::Versus => self.draw_versus(f, t),
            Scene::Play => {
                if let (Some(g), Some(sc)) = (&self.game, &self.scenery) {
                    draw_match(f, g, sc, &self.bank, t);
                    if self.paused {
                        f.shade_rect(0, 33, W, H - 33, 1, 2);
                        f.logo(128, 100, "PAUSE", 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
                        f.text_center(128, 140, "PUSH START", rgb(248, 248, 248), OUTLINE, true);
                    }
                    if self.mode == Mode::Demo {
                        f.rect(0, 212, W, 12, rgb(16, 16, 48));
                        if (t / 30) % 2 == 0 {
                            f.text_center(128, 214, "DEMO PLAY - PUSH START", rgb(248, 232, 64), OUTLINE, true);
                        }
                    }
                }
            }
            Scene::Continue => self.draw_continue(f, t),
            Scene::GameOver => {
                stripes(f, t, rgb(16, 8, 32), rgb(24, 16, 48));
                f.logo(128, 80, "GAME", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
                f.logo(128, 118, "OVER", 4, &grad6(rgb(216, 48, 64)), OUTLINE, rgb(0, 0, 0));
                f.text_center(128, 180, "BETTER LUCK NEXT TIME, DUDE", rgb(200, 200, 232), OUTLINE, false);
            }
            Scene::Champion => self.draw_champion(f, t),
        }
    }

    fn draw_title(&self, f: &mut Frame, t: u32) {
        f.copy_from(&self.title_bg.bg);
        self.title_bg.draw_ambient(f, t, 0.0, false);
        f.shade_rect(0, 0, W, 90, 3, 4);
        // logo drops in and bounces
        let st = self.t as i32;
        let drop = |target: i32, delay: i32| -> i32 {
            let k = st - delay;
            if k < 0 {
                -80
            } else if k < 20 {
                -80 + (target + 80) * k / 20
            } else if k < 30 {
                target - (6 - ((k - 25).abs() * 6 / 5))
            } else {
                target
            }
        };
        f.logo(122, drop(8, 0), "DODGE", 5, &SUNSET, OUTLINE, rgb(24, 8, 40));
        f.logo(138, drop(52, 8), "OR DIE!", 4, &grad6(rgb(232, 48, 64)), OUTLINE, rgb(24, 8, 40));
        if st > 36 {
            f.rect(0, 91, W, 11, rgb(24, 16, 56));
            f.hline(0, W - 1, 90, rgb(248, 176, 32));
            f.hline(0, W - 1, 102, rgb(248, 176, 32));
            f.text_grad_center(128, 93, "RADICAL BEACH DODGEBALL", &grad7(rgb(96, 224, 248)), OUTLINE, true);
        }
        // two players posing on the court
        let s1 = self.bank.get(TEAMS[0].players[0].style, Pose::WindUp);
        blit(f, s1, 70, 176, false, &palette(0, 0));
        draw_ball(f, 70 - 12 + 5, 176 - 30 + 7, 0.0, None);
        let s2 = self.bank.get(TEAMS[2].players[0].style, Pose::Catch);
        blit(f, s2, 186, 176, true, &palette(2, 0));
        // a ball flying between them
        let k = (t % 90) as f32 / 90.0;
        let bx = 80.0 + k * 96.0;
        let by = 150.0 - (k * std::f32::consts::PI).sin() * 30.0;
        f.shade_ellipse(bx as i32, 176, 3, 1, 5, 8);
        draw_ball(f, bx as i32, by as i32, t as f32 * 0.4, None);

        if !self.menu_open {
            if (t / 24) % 2 == 0 && st > 36 {
                f.text_grad_center(128, 188, "PUSH START", &grad7(rgb(248, 248, 248)), OUTLINE, true);
            }
        } else {
            f.shade_rect(72, 179, 112, 24, 1, 2);
            f.rect_outline(72, 179, 112, 24, rgb(248, 176, 32));
            for (k, s) in ["1 PLAYER", "2 PLAYERS"].iter().enumerate() {
                let y = 181 + k as i32 * 11;
                let sel = self.menu == k;
                let col = if sel { rgb(248, 232, 64) } else { rgb(184, 184, 216) };
                f.text_sh(100, y, s, col, OUTLINE, true);
                if sel {
                    draw_ball(f, 90, y + 3, t as f32 * 0.3, None);
                }
            }
        }
        // footer panel: scrolling ticker with the five D's, and the copyright
        f.rect(0, 200, W, 24, rgb(16, 16, 48));
        f.hline(0, W - 1, 199, rgb(248, 176, 32));
        let msg = "REMEMBER THE 5 D'S OF DODGEBALL:  DODGE, DUCK, DIP, DIVE ... AND DODGE!      ";
        let w = text_width(msg, false) + 6;
        let off = (t as i32 / 2) % w;
        for k in 0..3 {
            f.text_sh(k * w - off, 202, msg, rgb(248, 216, 48), OUTLINE, false);
        }
        f.text_center(128, 213, "(C) 1992 HELICO SOFT", rgb(248, 232, 200), rgb(96, 48, 32), false);
    }

    fn draw_select(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(16, 16, 72), rgb(24, 24, 96));
        f.text_grad_center(128, 6, "CHOOSE YOUR CREW", &grad7(rgb(248, 200, 48)), OUTLINE, true);
        for k in 0..4 {
            let (cx, cy) = (6 + (k % 2) as i32 * 124, 20 + (k / 2) as i32 * 92);
            let td = &TEAMS[k];
            let hover: Vec<usize> = (0..2).filter(|&p| (p == 0 || self.mode == Mode::Two) && self.cursor[p] == k).collect();
            f.rect(cx, cy, 120, 88, rgb(8, 8, 32));
            f.rect(cx + 2, cy + 2, 116, 12, shade(td.shirt.0, 1, 2));
            f.rect_outline(cx, cy, 120, 88, shade(td.shirt.0, 2, 3));
            f.text_grad(cx + 5, cy + 4, td.name, &grad7(td.acc.0), OUTLINE, false);
            // floor strip
            f.rect(cx + 2, cy + 64, 116, 6, shade(td.shirt.1, 1, 2));
            for slot in 0..3 {
                let active = !hover.is_empty();
                let pose = if active {
                    [Pose::Run1, Pose::Run2, Pose::Run3, Pose::Run4][((t / 6) as usize + slot) % 4]
                } else {
                    Pose::Stand
                };
                let s = self.bank.get(td.players[slot].style, pose);
                blit(f, s, cx + 13 + slot as i32 * 20, cy + 66, false, &palette(k, slot));
            }
            for (r, (lab, v)) in [("POW", td.power), ("SPD", td.speed), ("CAT", td.catch)].iter().enumerate() {
                let y = cy + 22 + r as i32 * 12;
                f.text(cx + 68, y, lab, rgb(200, 200, 232));
                for pip in 0..5 {
                    let c = if pip < *v { td.acc.0 } else { rgb(40, 40, 72) };
                    f.rect(cx + 87 + pip * 6, y, 6, 7, OUTLINE);
                    f.rect(cx + 88 + pip * 6, y + 1, 4, 5, c);
                }
            }
            f.text(cx + 5, cy + 76, td.court.name(), rgb(144, 152, 200));
            for &p in &hover {
                let col = if p == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
                let locked = self.locked[p];
                if locked || (t / 8) % 2 == 0 {
                    f.rect_outline(cx - 1, cy - 1, 122, 90, col);
                    f.rect_outline(cx, cy, 120, 88, col);
                }
                let tag = if p == 0 { "1P" } else { "2P" };
                let tx = if p == 0 { cx + 88 } else { cx + 103 };
                f.rect(tx - 1, cy + 59, 14, 9, col);
                f.text(tx, cy + 60, tag, rgb(16, 16, 48));
                if locked {
                    f.text_grad_center(cx + 60, cy + 40, "OK!", &grad7(col), OUTLINE, true);
                }
            }
        }
        if self.mode == Mode::Two {
            f.text_center(128, 208, "1P: ARROWS + , . /  (OR Z X C)", rgb(248, 232, 64), OUTLINE, false);
            f.text_center(128, 216, "2P: W A S D + F G H", rgb(96, 224, 248), OUTLINE, false);
        } else {
            f.text_center(128, 214, "ARROWS: PICK   Z: OK   X: BACK", rgb(200, 200, 232), OUTLINE, false);
        }
    }

    fn draw_versus(&self, f: &mut Frame, t: u32) {
        let (a, b) = match self.mode {
            Mode::One => (self.cursor[0], self.ladder.get(self.stage).copied().unwrap_or(1)),
            _ => (self.cursor[0], self.cursor[1]),
        };
        let (ta, tb) = (&TEAMS[a], &TEAMS[b]);
        // diagonal split, each half in team colors
        for y in 0..H {
            let split = 128 + (112 - y) / 3;
            for x in 0..W {
                let stripe = ((x + y + t as i32) / 6) % 2 == 0;
                let c = if x < split {
                    if stripe { shade(ta.shirt.1, 1, 2) } else { shade(ta.shirt.1, 2, 5) }
                } else if stripe {
                    shade(tb.shirt.1, 1, 2)
                } else {
                    shade(tb.shirt.1, 2, 5)
                };
                f.set(x, y, c);
            }
        }
        let slide = (40 - self.t as i32).max(0) * 6;
        for slot in 0..3 {
            let s = self.bank.get(ta.players[slot].style, if slot == 1 { Pose::Hold } else { Pose::Stand });
            blit_scaled(f, s, 28 + slot as i32 * 30 - slide, 150 + slot as i32 * 4, false, &palette(a, slot), 2);
            let s = self.bank.get(tb.players[slot].style, if slot == 1 { Pose::Hold } else { Pose::Stand });
            blit_scaled(f, s, 228 - slot as i32 * 30 + slide, 150 + slot as i32 * 4, true, &palette(b, slot), 2);
        }
        f.text_grad(8 - slide, 20, ta.name, &grad7(ta.shirt.0), OUTLINE, true);
        let wb = text_width(tb.name, true);
        f.text_grad(248 - wb + slide, 178, tb.name, &grad7(tb.shirt.0), OUTLINE, true);
        if self.t > 20 {
            let pulse = if self.t < 30 { 7 } else { 6 };
            f.logo(128, 44, "VS", pulse, &SUNSET, OUTLINE, rgb(8, 8, 24));
        }
        f.rect(0, 198, W, 26, rgb(8, 8, 32));
        f.hline(0, W - 1, 198, rgb(248, 176, 32));
        let court = tb.court.name();
        f.text_center(128, 202, court, rgb(248, 232, 200), OUTLINE, true);
        if self.mode == Mode::One {
            let s = format!("MATCH {} OF 3", self.stage + 1);
            f.text_center(128, 212, &s, rgb(160, 168, 216), OUTLINE, false);
        }
    }

    fn draw_continue(&self, f: &mut Frame, t: u32) {
        stripes(f, t, rgb(32, 8, 24), rgb(48, 16, 40));
        f.logo(128, 40, "CONTINUE?", 3, &SUNSET, OUTLINE, rgb(0, 0, 0));
        let n = format!("{}", self.count.max(0));
        f.logo(128, 90, &n, 6, &grad6(rgb(248, 232, 64)), OUTLINE, rgb(0, 0, 0));
        let team = self.cursor[0];
        for slot in 0..3 {
            let s = self.bank.get(TEAMS[team].players[slot].style, Pose::Down);
            blit(f, s, 88 + slot as i32 * 40, 180, slot % 2 == 1, &palette(team, slot));
        }
        if (t / 20) % 2 == 0 {
            f.text_center(128, 196, "PUSH Z TO GET BACK IN THE GAME", rgb(248, 248, 248), OUTLINE, false);
        }
    }

    fn draw_champion(&self, f: &mut Frame, t: u32) {
        f.copy_from(&self.title_bg.bg);
        self.title_bg.draw_ambient(f, t, 1.0, false);
        f.shade_rect(0, 0, W, 100, 2, 3);
        // trophy
        let (cx, cy) = (128, 104);
        let gold = rgb(248, 200, 48);
        let gold_d = rgb(184, 120, 24);
        f.shade_rect(0, 100, W, 10, 2, 3);
        f.fill_ellipse(cx, cy - 30, 20, 8, OUTLINE);
        f.fill_poly(&[(cx - 20, cy - 31), (cx + 20, cy - 31), (cx + 11, cy - 4), (cx - 11, cy - 4)], OUTLINE);
        f.rect(cx - 15, cy + 1, 30, 12, OUTLINE);
        f.circle_outline(cx - 20, cy - 22, 7, OUTLINE);
        f.circle_outline(cx + 20, cy - 22, 7, OUTLINE);
        f.fill_ellipse(cx, cy - 30, 18, 6, gold_d);
        f.fill_ellipse(cx, cy - 31, 15, 4, rgb(128, 72, 16));
        f.fill_poly(&[(cx - 18, cy - 30), (cx + 18, cy - 30), (cx + 10, cy - 6), (cx - 10, cy - 6)], gold);
        f.fill_poly(&[(cx + 4, cy - 30), (cx + 18, cy - 30), (cx + 10, cy - 6), (cx + 4, cy - 6)], gold_d);
        f.circle_outline(cx - 20, cy - 22, 6, gold);
        f.circle_outline(cx + 20, cy - 22, 6, gold_d);
        f.rect(cx - 3, cy - 6, 6, 8, gold_d);
        f.rect(cx - 12, cy + 2, 24, 6, gold);
        f.rect(cx - 14, cy + 8, 28, 4, rgb(96, 56, 32));
        if (t / 4) % 8 == 0 {
            f.set(cx - 10, cy - 26, rgb(248, 248, 248));
            f.set(cx - 9, cy - 26, rgb(248, 248, 248));
            f.set(cx - 10, cy - 25, rgb(248, 248, 248));
        }
        let team = self.cursor[0];
        for slot in 0..3 {
            let hop = (((t as i32 + slot as i32 * 11) % 30) - 15).abs();
            let y = 190 - (15 - hop).max(0) / 2;
            let pose = if hop < 8 { Pose::Victory } else { Pose::Stand };
            let x = [84, 128, 172][slot];
            f.shade_ellipse(x, 190, 7, 2, 5, 8);
            blit(f, self.bank.get(TEAMS[team].players[slot].style, pose), x, y, slot == 2, &palette(team, slot));
        }
        for &(x, y, _, c) in &self.confetti {
            f.rect(x as i32, y as i32, 2, 2, c);
        }
        f.logo(128, 8, "TOTALLY", 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
        f.logo(128, 34, "RADICAL!", 3, &SUNSET, OUTLINE, rgb(8, 8, 24));
        let s = format!("{} ARE THE CHAMPS!", TEAMS[team].short);
        f.text_grad_center(128, 122, &s, &grad7(rgb(96, 224, 248)), OUTLINE, true);
        f.text_center(128, 206, "THANKS FOR PLAYING", rgb(248, 248, 248), OUTLINE, true);
    }
}

fn stripes(f: &mut Frame, t: u32, a: u32, b: u32) {
    for y in 0..H {
        for x in 0..W {
            let k = ((x + y + (t / 2) as i32) / 8) % 2;
            f.set(x, y, if k == 0 { a } else { b });
        }
    }
}

/// Integer-scaled sprite (used for the big versus screen).
pub fn blit_scaled(f: &mut Frame, s: &Sprite, x: i32, y: i32, flip: bool, pal: &Palette, k: i32) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let v = s.px[(sy * s.w + sx) as usize];
            if v == T {
                continue;
            }
            let px = if flip { x + (OX - sx) * k } else { x + (sx - OX) * k };
            f.rect(px, y + (sy - OY) * k, k, k, pal[v as usize]);
        }
    }
}
