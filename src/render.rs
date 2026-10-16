//! Draws a match: court, crowd, players, ball, effects, HUD and banners.

use crate::court::Scenery;
use crate::game::*;
use crate::gfx::*;
use crate::sprites::{self, blit, blit_ghost, canvas_to_screen, Palette, Pose, Sprite, SpriteBank};
use crate::teams::{flash_palette, ghost_palette, palette, OUTLINE};

pub const SUNSET: [u32; 6] =
    [rgb(248, 248, 168), rgb(248, 224, 72), rgb(248, 176, 32), rgb(248, 120, 32), rgb(232, 64, 48), rgb(176, 32, 88)];

pub fn grad7(c: u32) -> [u32; 7] {
    [mix(c, 0xFFFFFF, 0.7), mix(c, 0xFFFFFF, 0.35), c, c, shade(c, 7, 8), shade(c, 3, 4), shade(c, 5, 8)]
}

pub fn grad6(c: u32) -> [u32; 6] {
    [mix(c, 0xFFFFFF, 0.6), mix(c, 0xFFFFFF, 0.25), c, shade(c, 7, 8), shade(c, 3, 4), shade(c, 3, 5)]
}

const BALL: [&str; 8] = ["..KKKK..", ".KRRRRK.", "KRWRRRrK", "KRRRRRrK", "KRRRRRrK", "KRRRrrrK", ".KrrrrK.", "..KKKK.."];

pub fn draw_ball(f: &mut Frame, x: i32, y: i32, spin: f32, hot: Option<u32>) {
    let (r, d) = match hot {
        Some(t) => {
            let cyc = [rgb(248, 248, 120), rgb(248, 168, 32), rgb(248, 248, 248), rgb(248, 96, 32)];
            (cyc[(t / 2 % 4) as usize], rgb(232, 64, 32))
        }
        None => (rgb(232, 56, 48), rgb(152, 24, 40)),
    };
    let seam = (spin as i32).rem_euclid(6) + 1;
    for (yy, row) in BALL.iter().enumerate() {
        for (xx, ch) in row.bytes().enumerate() {
            let c = match ch {
                b'K' => OUTLINE,
                b'R' => {
                    if xx as i32 == seam && (1..7).contains(&yy) {
                        d
                    } else {
                        r
                    }
                }
                b'r' => d,
                b'W' => rgb(248, 248, 248),
                _ => continue,
            };
            f.set(x - 4 + xx as i32, y - 4 + yy as i32, c);
        }
    }
}

pub fn heart(f: &mut Frame, x: i32, y: i32, full: bool) {
    let rows = [".#.#.", "#####", "#####", ".###.", "..#.."];
    let (c, hl) = if full { (rgb(248, 56, 72), rgb(248, 200, 200)) } else { (rgb(56, 48, 80), rgb(56, 48, 80)) };
    for (yy, row) in rows.iter().enumerate() {
        for (xx, ch) in row.bytes().enumerate() {
            if ch == b'#' {
                let col = if yy == 1 && xx == 1 { hl } else { c };
                f.set(x + xx as i32, y + yy as i32, col);
            }
        }
    }
}

/// Draw part of a sprite (used for the head icons in the HUD).
pub fn blit_region(f: &mut Frame, s: &Sprite, sx0: i32, sy0: i32, w: i32, h: i32, x: i32, y: i32, pal: &Palette, flip: bool) {
    for yy in 0..h {
        for xx in 0..w {
            let v = s.get(sx0 + xx, sy0 + yy);
            if v != sprites::T {
                let px = if flip { x + w - 1 - xx } else { x + xx };
                f.set(px, y + yy, pal[v as usize]);
            }
        }
    }
}

pub fn pose_for(m: &Match, i: usize) -> Pose {
    let p = &m.players[i];
    let holding = m.holder() == Some(i);
    let run_frame = |a: f32| [Pose::Run1, Pose::Run2, Pose::Run3, Pose::Run4][(a as usize) % 4];
    match p.state {
        PState::Normal => {
            if p.z > 0.0 {
                Pose::Jump
            } else if p.moving() {
                run_frame(p.anim)
            } else if holding {
                Pose::Hold
            } else {
                Pose::Stand
            }
        }
        PState::WindUp => {
            if p.z > 0.0 {
                Pose::JumpWind
            } else {
                Pose::WindUp
            }
        }
        PState::Throw => Pose::Throw,
        PState::Catch => Pose::Catch,
        PState::Hit => Pose::Hit,
        PState::Down => Pose::Down,
        PState::Duck => Pose::Duck,
        PState::Dive => Pose::Dive,
        PState::Out => Pose::Stand,
        PState::Victory => {
            if p.z > 0.0 {
                Pose::Victory
            } else {
                Pose::Stand
            }
        }
    }
}

enum Item {
    Player(usize),
    Ball,
    Coach(usize),
    Wrench,
}

pub fn draw_wrench(f: &mut Frame, x: i32, y: i32, spin: f32) {
    let (c, s) = (spin.cos(), spin.sin());
    let (x0, y0) = (x - (c * 5.0) as i32, y - (s * 5.0) as i32);
    let (x1, y1) = (x + (c * 5.0) as i32, y + (s * 5.0) as i32);
    f.thick_line(x0, y0 + 1, x1, y1 + 1, 2, OUTLINE);
    f.thick_line(x0, y0, x1, y1, 2, rgb(184, 192, 208));
    f.fill_ellipse(x1, y1, 2, 2, rgb(216, 224, 232));
    f.set(x1 + (c * 2.0) as i32, y1 + (s * 2.0) as i32, OUTLINE);
    f.set(x0, y0, rgb(248, 248, 248));
}

fn coach_palette(team: usize) -> Palette {
    let mut pal = palette(team, 0);
    let td = &crate::teams::TEAMS[team];
    pal[sprites::SKIN as usize] = rgb(232, 176, 136);
    pal[sprites::SKIN_D as usize] = rgb(184, 120, 88);
    pal[sprites::HAIR as usize] = rgb(200, 200, 208);
    pal[sprites::HAIR_D as usize] = rgb(136, 136, 152);
    pal[sprites::SHIRT as usize] = shade(td.shirt.0, 3, 4);
    pal[sprites::SHIRT_D as usize] = shade(td.shirt.1, 3, 4);
    pal[sprites::PANTS as usize] = rgb(88, 80, 72);
    pal[sprites::PANTS_D as usize] = rgb(56, 48, 48);
    pal[sprites::ACC as usize] = rgb(248, 248, 248);
    pal[sprites::ACC_D as usize] = rgb(160, 160, 176);
    pal
}

fn draw_coach(f: &mut Frame, m: &Match, bank: &SpriteBank, team: usize, t: u32) {
    let c = &m.coaches[team];
    let (x, y) = coach_pos(team);
    let (sx, sy) = proj(x, y, 0.0);
    let pose = if c.warn > 0 {
        Pose::WindUp
    } else if c.throw_t > 0 {
        Pose::Throw
    } else if m.phase == Phase::Play && (t / 40) % 4 == 0 {
        Pose::Victory
    } else {
        Pose::Stand
    };
    let flip = team == 1;
    f.shade_ellipse(sx, sy, 7, 2, 5, 8);
    blit(f, bank.get(2, pose), sx, sy, flip, &coach_palette(m.sides[team].team));
    if c.warn > 0 {
        let (hx, hy) = sprites::hand(pose);
        let (wx, wy) = canvas_to_screen(hx, hy, sx, sy, flip);
        draw_wrench(f, wx, wy, 0.8);
    }
}

pub fn draw_match(f: &mut Frame, m: &Match, sc: &Scenery, bank: &SpriteBank, t: u32) {
    f.copy_from(&sc.bg);
    sc.draw_ambient(f, t, m.excite);

    // shadows first, they sit on the floor under everything
    for p in &m.players {
        if p.state == PState::Out {
            continue;
        }
        let (sx, sy) = proj(p.x, p.y, 0.0);
        let r = (7 - (p.z / 8.0) as i32).max(3);
        f.shade_ellipse(sx, sy, r, 2, 5, 8);
    }
    let b = &m.ball;
    if !matches!(b.state, BallState::Held(_)) {
        let (sx, sy) = proj(b.x, b.y, 0.0);
        let r = (4 - (b.z / 14.0) as i32).max(2);
        f.shade_ellipse(sx, sy, r, 1, 5, 8);
    }

    let mut items: Vec<(f32, Item)> = m.players.iter().enumerate().filter(|(_, p)| p.state != PState::Out).map(|(i, p)| (p.y, Item::Player(i))).collect();
    if !matches!(b.state, BallState::Held(_)) {
        items.push((b.y + 0.1, Item::Ball));
    }
    for team in 0..2 {
        items.push((coach_pos(team).1, Item::Coach(team)));
    }
    if let Some(w) = &m.wrench {
        let (sx, sy) = proj(w.x, w.y, 0.0);
        f.shade_ellipse(sx, sy, 3, 1, 5, 8);
        items.push((w.y + 0.2, Item::Wrench));
    }
    items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for (_, it) in &items {
        match *it {
            Item::Player(i) => draw_player(f, m, bank, i, t),
            Item::Coach(team) => draw_coach(f, m, bank, team, t),
            Item::Wrench => {
                if let Some(w) = &m.wrench {
                    let (sx, sy) = proj(w.x, w.y, w.z);
                    draw_wrench(f, sx, sy, w.spin);
                }
            }
            Item::Ball => {
                if b.state == BallState::Flying && b.shot == Shot::Super {
                    let n = b.trail.len().max(1);
                    let pts: Vec<(i32, i32, i32)> = b
                        .trail
                        .iter()
                        .enumerate()
                        .map(|(k, &(x, y, z))| {
                            let (sx, sy) = proj(x, y, z);
                            (sx, sy, 1 + (k * 4 / n) as i32)
                        })
                        .collect();
                    for &(sx, sy, r) in &pts {
                        f.fill_ellipse(sx, sy, r + 1, r + 1, OUTLINE);
                    }
                    for (k, &(sx, sy, r)) in pts.iter().enumerate() {
                        let c = [rgb(176, 32, 48), rgb(232, 72, 32), rgb(248, 160, 32), rgb(248, 232, 96)][(k * 4 / n).min(3)];
                        f.fill_ellipse(sx, sy, r, r, c);
                    }
                    let (sx, sy) = proj(b.x, b.y, b.z);
                    let back = if b.vx > 0.0 { -1 } else { 1 };
                    for dy in [-4, 4] {
                        let wob = ((t as i32 + dy) / 2) % 3;
                        f.hline(sx + back * (6 + wob), sx + back * (16 + wob), sy + dy, rgb(248, 232, 96));
                    }
                } else if b.state == BallState::Flying && b.shot == Shot::Dash {
                    for (k, &(x, y, z)) in b.trail.iter().enumerate().step_by(2) {
                        let (sx, sy) = proj(x, y, z);
                        if k > 1 {
                            f.hline(sx - 2, sx + 2, sy, rgb(248, 248, 248));
                        }
                    }
                }
                let (sx, sy) = proj(b.x, b.y, b.z);
                let hot = (b.state == BallState::Flying && b.shot == Shot::Super).then_some(t);
                draw_ball(f, sx, sy, b.spin, hot);
            }
        }
    }

    // ghosts float above everything
    for (i, p) in m.players.iter().enumerate() {
        if p.state == PState::Out && p.ghost < 140.0 {
            let (sx, sy) = proj(p.x, p.y, p.ghost);
            let s = bank.get(crate::teams::TEAMS[m.sides[p.team].team].players[p.slot].style, Pose::Stand);
            let sway = ((p.ghost * 0.08).sin() * 3.0) as i32;
            blit_ghost(f, s, sx + sway, sy, p.face < 0.0, &ghost_palette(), t);
            f.circle_outline(sx + sway, sy - 32, 4, rgb(248, 232, 96));
            f.hline(sx + sway - 3, sx + sway + 3, sy - 33, rgb(248, 232, 96));
            let wing = if (t / 6) % 2 == 0 { 0 } else { 1 };
            for side in [-1, 1] {
                f.line(sx + sway + side * 5, sy - 20, sx + sway + side * 10, sy - 24 - wing, rgb(248, 248, 248));
                f.line(sx + sway + side * 5, sy - 18, sx + sway + side * 9, sy - 20 - wing, rgb(248, 248, 248));
            }
            let _ = i;
        }
    }

    for fx in &m.fx {
        let (sx, sy) = proj(fx.x, fx.y, fx.z);
        match fx.kind {
            FxKind::Dust => {
                let r = 1 + (fx.life * 3 / fx.max);
                let c = dust_color(sc.court);
                f.fill_ellipse(sx, sy, r, r.max(1) - 0, c);
            }
            FxKind::Star => {
                f.set(sx, sy, fx.color);
                f.set(sx - 1, sy, fx.color);
                f.set(sx + 1, sy, fx.color);
                f.set(sx, sy - 1, fx.color);
                f.set(sx, sy + 1, fx.color);
            }
            FxKind::Spark => {
                f.set(sx, sy, rgb(248, 248, 248));
                f.set(sx + 1, sy, rgb(120, 248, 120));
            }
            FxKind::Impact => {
                // starburst: rays shrink after the first frames
                let age = fx.max - fx.life;
                let len = if age < 3 { 7 } else { 7 - (age - 2) * 2 };
                if len > 1 {
                    for k in 0..8 {
                        let a = k as f32 / 8.0 * std::f32::consts::TAU + 0.2;
                        let l = if k % 2 == 0 { len } else { len * 2 / 3 };
                        let (ex, ey) = (sx + (a.cos() * l as f32) as i32, sy + (a.sin() * l as f32) as i32);
                        f.thick_line(sx, sy, ex, ey, 3, OUTLINE);
                    }
                    for k in 0..8 {
                        let a = k as f32 / 8.0 * std::f32::consts::TAU + 0.2;
                        let l = if k % 2 == 0 { len } else { len * 2 / 3 };
                        let (ex, ey) = (sx + (a.cos() * l as f32) as i32, sy + (a.sin() * l as f32) as i32);
                        f.thick_line(sx, sy, ex, ey, 1, rgb(248, 232, 64));
                        f.set(ex, ey, rgb(248, 136, 32));
                    }
                    f.fill_ellipse(sx, sy, 2, 2, rgb(248, 248, 248));
                }
            }
            FxKind::Sweat => {
                f.set(sx, sy, rgb(160, 216, 248));
                f.set(sx, sy + 1, rgb(96, 168, 232));
                f.set(sx - 1, sy + 1, rgb(160, 216, 248));
            }
        }
    }

    for p in &m.popups {
        let a = p.life > 8 || (p.life / 2) % 2 == 0;
        if a {
            let w = text_width(p.text, true);
            f.text_grad((p.x as i32) - w / 2, p.y as i32, p.text, &grad7(p.color), OUTLINE, true);
        }
    }

    // screen shake moves the world, not the HUD
    if m.shake > 0 {
        let dx = if (m.shake / 2) % 2 == 0 { m.shake.min(4) / 2 + 1 } else { -(m.shake.min(4) / 2 + 1) };
        let src = f.clone();
        for y in 32..H {
            for x in 0..W {
                f.set(x, y, src.get((x - dx).clamp(0, W - 1), y));
            }
        }
    }

    if m.flash > 0 {
        f.shade_rect(0, 33, W, H - 33, 5, 4);
    }
    draw_hud(f, m, bank, t);
    if let Some(bn) = &m.banner {
        draw_banner(f, bn);
    }
    if m.quote > 0 {
        draw_quote(f, m, bank);
    }
}

/// The coach's famous line, in a classic dialog box.
fn draw_quote(f: &mut Frame, m: &Match, bank: &SpriteBank) {
    let y = 196;
    f.rect(4, y, 248, 26, rgb(16, 16, 48));
    f.rect_outline(4, y, 248, 26, rgb(248, 248, 248));
    f.rect_outline(5, y + 1, 246, 24, rgb(96, 104, 168));
    let pal = coach_palette(m.sides[m.quote_team].team);
    blit_region(f, bank.get(2, Pose::Stand), 6, 0, 12, 12, 9, y + 7, &pal, false);
    let shown = ((170 - m.quote) * 2).max(0) as usize;
    let l1 = "IF YOU CAN DODGE A WRENCH,";
    let l2 = "YOU CAN DODGE A BALL!";
    let a: String = l1.chars().take(shown).collect();
    let b: String = l2.chars().take(shown.saturating_sub(l1.len())).collect();
    f.text_sh(26, y + 4, "COACH:", rgb(248, 176, 32), OUTLINE, false);
    f.text_sh(66, y + 4, &a, rgb(248, 248, 248), OUTLINE, false);
    f.text_sh(66, y + 14, &b, rgb(248, 248, 248), OUTLINE, false);
}

fn dust_color(c: crate::teams::Court) -> u32 {
    match c {
        crate::teams::Court::Beach => rgb(248, 224, 168),
        crate::teams::Court::Slopes => rgb(248, 248, 248),
        crate::teams::Court::Boardwalk => rgb(192, 152, 112),
        crate::teams::Court::Skatepark => rgb(216, 216, 224),
    }
}

fn draw_player(f: &mut Frame, m: &Match, bank: &SpriteBank, i: usize, t: u32) {
    let p = &m.players[i];
    if p.invuln > 0 && (p.invuln / 3) % 2 == 0 {
        return;
    }
    let team = m.sides[p.team].team;
    let def = &crate::teams::TEAMS[team].players[p.slot];
    let pose = pose_for(m, i);
    let s = bank.get(def.style, pose);
    let (sx, sy) = proj(p.x, p.y, p.z);
    let flip = p.face < 0.0;
    let base = palette(team, p.slot);
    let pal = if p.flash > 4 { flash_palette(&base) } else { base };
    if p.fired_up {
        for k in 0..6u32 {
            let ph = (t + k * 7) % 24;
            let fx = sx - 8 + ((k * 37 + t / 3) % 17) as i32;
            let fy = sy - 2 - ph as i32;
            let c = [rgb(248, 232, 96), rgb(248, 152, 32), rgb(232, 64, 32)][(ph / 8) as usize];
            f.set(fx, fy, c);
            f.set(fx, fy - 1, c);
        }
    }
    blit(f, s, sx, sy, flip, &pal);

    if m.holder() == Some(i) {
        let (hx, hy) = sprites::hand(pose);
        let (bx, by) = canvas_to_screen(hx, hy, sx, sy, flip);
        draw_ball(f, bx + if flip { -1 } else { 1 }, by - 1, m.ball.spin, None);
    }
    if p.state == PState::Down && p.hp > 0 {
        // dizzy stars
        for k in 0..3 {
            let a = t as f32 * 0.15 + k as f32 * 2.1;
            let x = sx + if flip { 6 } else { -6 } + (a.cos() * 6.0) as i32;
            let y = sy - 9 + (a.sin() * 2.0) as i32;
            f.set(x, y, rgb(248, 248, 96));
            f.set(x + 1, y, rgb(248, 248, 96));
        }
    }
    // marker over the human-controlled player
    for side in 0..2 {
        if let Ctrl::Human(pi) = m.sides[side].ctrl {
            if m.sides[side].control == i && p.alive() && m.phase != Phase::MatchOver {
                let bob = ((t / 8) % 2) as i32;
                let col = if pi == 0 { rgb(248, 232, 64) } else { rgb(96, 224, 248) };
                let label = if pi == 0 { "1P" } else { "2P" };
                let y = sy - 44 - bob;
                f.text_grad(sx - 7, y, label, &grad7(col), OUTLINE, true);
                for k in 0..3 {
                    f.hline(sx - 3 + k, sx + 3 - k, y + 8 + k, OUTLINE);
                    f.hline(sx - 2 + k, sx + 2 - k, y + 8 + k, col);
                }
            }
        }
    }
}

pub fn draw_hud(f: &mut Frame, m: &Match, bank: &SpriteBank, t: u32) {
    f.rect(0, 0, W, 31, rgb(16, 16, 48));
    f.hline(0, W - 1, 31, rgb(248, 176, 32));
    f.hline(0, W - 1, 32, rgb(176, 32, 88));
    for side in 0..2 {
        let team = m.sides[side].team;
        let td = &crate::teams::TEAMS[team];
        let base = if side == 0 { 4 } else { 152 };
        let name = td.short;
        let nx = if side == 0 { base } else { 252 - text_width(name, true) };
        f.text_grad(nx, 3, name, &grad7(td.shirt.0), OUTLINE, true);
        for slot in 0..TEAM_SIZE {
            let i = side * TEAM_SIZE + slot;
            let p = &m.players[i];
            let x = base + slot as i32 * 34;
            let pal = if p.state == PState::Out { ghost_palette() } else { palette(team, slot) };
            let s = bank.get(td.players[slot].style, Pose::Stand);
            let ctl = matches!(m.sides[side].ctrl, Ctrl::Human(_)) && m.sides[side].control == i;
            if ctl && (t / 10) % 2 == 0 {
                f.rect_outline(x - 1, 12, 34, 17, rgb(248, 232, 64));
            }
            blit_region(f, s, 6, 0, 12, 12, x, 14, &pal, side == 1);
            for h in 0..MAX_HP {
                heart(f, x + 13 + (h % 2) * 6, 15 + (h / 2) * 6, h < p.hp);
            }
            if p.state == PState::Out {
                f.line(x + 1, 15, x + 10, 25, rgb(248, 56, 72));
                f.line(x + 10, 15, x + 1, 25, rgb(248, 56, 72));
            }
        }
        // round wins
        for w in 0..ROUNDS_TO_WIN {
            let x = if side == 0 { 108 - w as i32 * 9 } else { 148 + w as i32 * 9 };
            let won = m.sides[side].wins > w;
            if won {
                draw_ball(f, x, 6, 0.0, None);
            } else {
                f.fill_ellipse(x, 6, 3, 3, rgb(48, 48, 88));
            }
        }
    }
    let secs = (m.time.max(0) + 59) / 60;
    let txt = format!("{:02}", secs);
    let low = secs <= 10 && m.phase == Phase::Play;
    let col = if low && (t / 8) % 2 == 0 { rgb(248, 56, 56) } else { rgb(248, 224, 72) };
    f.text_center(128, 2, "TIME", rgb(160, 168, 216), OUTLINE, false);
    f.logo(128, 8, &txt, 2, &grad6(col), OUTLINE, rgb(8, 8, 24));
}

pub fn draw_banner(f: &mut Frame, b: &Banner) {
    let n = b.text.chars().count() as i32;
    let scale = if b.scale > 0 { b.scale } else if n * 18 < 236 { 3 } else { 2 };
    let target = 128;
    let (x, y) = if let Some(y) = b.y {
        (if b.t < 6 { target + (6 - b.t) * 40 } else { target }, y)
    } else if b.pop {
        (target, 120 - 7 * scale / 2 - if b.t < 4 { 4 - b.t } else { 0 })
    } else {
        let x = if b.t < 10 { target + (10 - b.t) * 30 } else { target };
        (x, 120 - 7 * scale / 2 + if (10..16).contains(&b.t) { (16 - b.t) / 2 } else { 0 })
    };
    let grad: Vec<u32> = if b.color == rgb(248, 248, 248) { SUNSET.to_vec() } else { grad6(b.color).to_vec() };
    f.logo(x, y, &b.text, scale, &grad, OUTLINE, rgb(8, 8, 24));
}
