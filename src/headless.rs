//! Headless tools: render scripted screenshots to PNG, and simulate many
//! CPU-vs-CPU matches to check game balance. No window needed.

use crate::app::{App, Mode, Scene};
use crate::court::Scenery;
use crate::game::*;
use crate::gfx::*;
use crate::render::draw_match;
use crate::sprites::{blit, ALL_POSES};
use crate::teams::*;
use std::path::Path;

pub fn save_png(f: &Frame, path: &Path, scale: usize) {
    let (w, h) = (W as usize * scale, H as usize * scale);
    let mut data = vec![0u8; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let c = f.px[(y / scale) * W as usize + x / scale];
            let i = (y * w + x) * 3;
            data[i] = (c >> 16) as u8;
            data[i + 1] = (c >> 8) as u8;
            data[i + 2] = c as u8;
        }
    }
    write_rgb(path, w, h, &data);
}

fn write_rgb(path: &Path, w: usize, h: usize, data: &[u8]) {
    let file = std::fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(data).unwrap();
}

/// Tile several frames into one image (to judge animation).
fn save_sheet(frames: &[Frame], cols: usize, path: &Path, scale: usize) {
    let rows = frames.len().div_ceil(cols);
    let (fw, fh) = (W as usize * scale, H as usize * scale);
    let (w, h) = (fw * cols, fh * rows);
    let mut data = vec![0u8; w * h * 3];
    for (k, f) in frames.iter().enumerate() {
        let (ox, oy) = ((k % cols) * fw, (k / cols) * fh);
        for y in 0..fh {
            for x in 0..fw {
                let c = f.px[(y / scale) * W as usize + x / scale];
                let i = ((oy + y) * w + ox + x) * 3;
                data[i] = (c >> 16) as u8;
                data[i + 1] = (c >> 8) as u8;
                data[i + 2] = c as u8;
            }
        }
    }
    write_rgb(path, w, h, &data);
}

fn press(app: &mut App, a: bool, dir: Option<&str>) {
    let mut p = Pad { a, ..Pad::default() };
    match dir {
        Some("right") => p.right = true,
        Some("left") => p.left = true,
        Some("down") => p.down = true,
        _ => {}
    }
    app.update([p, Pad::default()], false);
    app.update([Pad::default(); 2], false);
}

fn run(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update([Pad::default(); 2], false);
    }
}

fn snap(app: &App, dir: &Path, name: &str) {
    let mut f = Frame::new();
    app.draw(&mut f);
    save_png(&f, &dir.join(name), 3);
    println!("wrote {}", name);
}

pub fn shots(out: &str) {
    let dir = Path::new(out);
    std::fs::create_dir_all(dir).unwrap();
    let mut app = App::new(42);

    run(&mut app, 70);
    snap(&app, dir, "01_title.png");
    press(&mut app, true, None);
    run(&mut app, 8);
    snap(&app, dir, "02_title_menu.png");
    press(&mut app, true, None);
    run(&mut app, 30);
    snap(&app, dir, "03_select.png");
    press(&mut app, false, Some("down"));
    run(&mut app, 5);
    press(&mut app, true, None);
    run(&mut app, 14);
    run(&mut app, 40);
    snap(&app, dir, "04_versus.png");
    run(&mut app, 130);
    run(&mut app, 30);
    snap(&app, dir, "05_round_intro.png");

    // every court with a CPU match in progress
    for (k, court) in [Court::Beach, Court::Boardwalk, Court::Slopes, Court::Skatepark].iter().enumerate() {
        app.mode = Mode::Demo;
        app.start_match([k, (k + 1) % 4], [Ctrl::Cpu, Ctrl::Cpu], [0.7, 0.7], *court);
        run_play(&mut app, 420);
        snap(&app, dir, &format!("06_court_{}.png", k));
    }

    // a long demo match: periodic shots and shots at key moments
    app.start_match([0, 2], [Ctrl::Cpu, Ctrl::Cpu], [0.65, 0.65], Court::Beach);
    let mut periodic = 0;
    let (mut hits, mut catches, mut supers) = (0, 0, 0);
    let (mut nh, mut nc, mut ns) = (0, 0, 0);
    let mut sheet: Vec<Frame> = Vec::new();
    let mut sheet_from: Option<u32> = None;
    for frame in 0..20000u32 {
        app.update([Pad::default(); 2], false);
        let Some(g) = app.game.as_ref() else { break };
        let st = g.stats;
        if st.hits > hits && nh < 4 {
            nh += 1;
            run_play(&mut app, 2);
            snap(&app, dir, &format!("07_hit_{}.png", nh));
        }
        if st.catches > catches && nc < 3 {
            nc += 1;
            snap(&app, dir, &format!("08_catch_{}.png", nc));
        }
        if st.supers > supers && ns < 3 {
            ns += 1;
            run_play(&mut app, 3);
            snap(&app, dir, &format!("09_super_{}.png", ns));
            if sheet_from.is_none() {
                sheet_from = Some(frame);
            }
        }
        let g = app.game.as_ref().unwrap();
        hits = g.stats.hits;
        catches = g.stats.catches;
        supers = g.stats.supers;
        if sheet_from.is_some() && sheet.len() < 12 && frame % 3 == 0 {
            let mut f = Frame::new();
            app.draw(&mut f);
            sheet.push(f);
        }
        if frame % 150 == 75 && periodic < 16 {
            periodic += 1;
            snap(&app, dir, &format!("10_play_{:02}.png", periodic));
        }
        if app.scene != Scene::Play {
            break;
        }
        if app.game.as_ref().map(|g| g.finished() && g.phase_t == 60).unwrap_or(false) {
            snap(&app, dir, "11_match_over.png");
        }
    }
    if !sheet.is_empty() {
        save_sheet(&sheet, 4, &dir.join("12_super_sequence.png"), 2);
        println!("wrote 12_super_sequence.png");
    }

    // sprite sheet: every pose for every player
    let scale = 3usize;
    let cols = ALL_POSES.len();
    let (cw, ch) = (26usize, 34usize);
    let (w, h) = (cols * cw * scale, 12 * ch * scale);
    let mut data = vec![0u8; w * h * 3];
    for row in 0..12 {
        let (team, slot) = (row / 3, row % 3);
        let mut f = Frame::new();
        f.clear(rgb(96, 128, 112));
        // draw into a temp frame in chunks of 9 poses (frame is 256 wide)
        for (k, pose) in ALL_POSES.iter().enumerate() {
            f.clear(rgb(96, 128, 112));
            let s = app.bank.get(TEAMS[team].players[slot].style, *pose);
            blit(&mut f, s, 13, 32, false, &palette(team, slot));
            for y in 0..ch * scale {
                for x in 0..cw * scale {
                    let c = f.get((x / scale) as i32, (y / scale) as i32);
                    let i = ((row * ch * scale + y) * w + k * cw * scale + x) * 3;
                    data[i] = (c >> 16) as u8;
                    data[i + 1] = (c >> 8) as u8;
                    data[i + 2] = c as u8;
                }
            }
        }
    }
    write_rgb(&dir.join("13_sprites.png"), w, h, &data);
    println!("wrote 13_sprites.png");

    // end screens
    app.cursor[0] = 0;
    app.mode = Mode::One;
    app.ladder = vec![1, 2, 3];
    app.stage = 3;
    app.scene = Scene::Champion;
    app.t = 200;
    run(&mut app, 100);
    snap(&app, dir, "14_champion.png");
    app.scene = Scene::Continue;
    app.t = 0;
    app.count = 9;
    run(&mut app, 70);
    snap(&app, dir, "15_continue.png");

    // 2P select
    let mut app2 = App::new(7);
    run(&mut app2, 30);
    press(&mut app2, true, None);
    press(&mut app2, false, Some("down"));
    press(&mut app2, true, None);
    run(&mut app2, 20);
    snap(&app2, dir, "16_select_2p.png");

    // the five D's chant and the coach's wrench (an idle human falls behind)
    let mut app3 = App::new(11);
    app3.mode = Mode::One;
    app3.start_match([0, 1], [Ctrl::Human(0), Ctrl::Cpu], [0.5, 0.6], Court::Beach);
    run_play(&mut app3, 84);
    snap(&app3, dir, "18_chant_duck.png");
    let mut seen = 0;
    for _ in 0..60 * 120 {
        run_play(&mut app3, 1);
        let Some(g) = app3.game.as_ref() else { break };
        if g.quote == 120 || g.wrench.as_ref().map(|w| w.t == 14).unwrap_or(false) {
            seen += 1;
            snap(&app3, dir, &format!("19_wrench_{}.png", seen));
            if seen >= 4 {
                break;
            }
        }
    }

    // static render check of a scene without the app wrapper
    let sc = Scenery::new(Court::Slopes, 1);
    let m = Match::new([2, 3], [Ctrl::Human(0), Ctrl::Cpu], [0.5, 0.5], Court::Slopes, 3);
    let mut f = Frame::new();
    draw_match(&mut f, &m, &sc, &app.bank, 0);
    save_png(&f, &dir.join("17_slopes_start.png"), 3);
    println!("wrote 17_slopes_start.png");
}

fn run_play(app: &mut App, n: usize) {
    for _ in 0..n {
        if app.scene != Scene::Play {
            return;
        }
        app.update([Pad::default(); 2], false);
    }
}

pub fn sim(n: usize) {
    let mut rng = Rng(12345);
    let mut tot = Stats::default();
    let mut matches = 0;
    let mut frames_total = 0u64;
    let mut team_wins = [0u32; 4];
    let mut team_games = [0u32; 4];
    let mut better_skill_won = 0;
    let mut skill_games = 0;
    for k in 0..n {
        let a = rng.int(4) as usize;
        let b = (a + 1 + rng.int(3) as usize) % 4;
        let sa = rng.range(0.3, 0.9);
        let sb = rng.range(0.3, 0.9);
        let mut m = Match::new([a, b], [Ctrl::Cpu, Ctrl::Cpu], [sa, sb], TEAMS[b].court, k as u64 + 1);
        let mut f = 0u64;
        while !m.finished() && f < 60 * 60 * 20 {
            m.update(&[Pad::default(); 2]);
            m.sfx.clear();
            f += 1;
        }
        frames_total += f;
        let s = m.stats;
        tot.throws += s.throws;
        tot.hits += s.hits;
        tot.catches += s.catches;
        tot.supers += s.supers;
        tot.dashes += s.dashes;
        tot.passes += s.passes;
        tot.dodges += s.dodges;
        tot.rounds += s.rounds;
        tot.timeouts += s.timeouts;
        tot.round_frames += s.round_frames;
        if let Some(w) = m.winner {
            matches += 1;
            team_games[a] += 1;
            team_games[b] += 1;
            team_wins[[a, b][w]] += 1;
            if (sa - sb).abs() > 0.2 {
                skill_games += 1;
                if (w == 0) == (sa > sb) {
                    better_skill_won += 1;
                }
            }
        }
    }
    let r = tot.rounds.max(1) as f32;
    println!("matches finished: {}/{}", matches, n);
    println!("avg match length: {:.1}s", frames_total as f32 / n as f32 / 60.0);
    println!("avg rounds/match: {:.2}", tot.rounds as f32 / n as f32);
    println!("avg round length: {:.1}s (playing time)", tot.round_frames as f32 / r / 60.0);
    println!("per round: throws {:.1}, hits {:.1}, catches {:.1}, supers {:.1}, dashes {:.1}, passes {:.1}, dodged {:.1}",
        tot.throws as f32 / r, tot.hits as f32 / r, tot.catches as f32 / r, tot.supers as f32 / r,
        tot.dashes as f32 / r, tot.passes as f32 / r, tot.dodges as f32 / r);
    println!("hit rate {:.0}%, catch rate {:.0}%", 100.0 * tot.hits as f32 / tot.throws.max(1) as f32,
        100.0 * tot.catches as f32 / tot.throws.max(1) as f32);
    println!("rounds decided by time: {:.0}%", 100.0 * tot.timeouts as f32 / r);
    for t in 0..4 {
        println!("  {:<15} win {:.0}% of {}", TEAMS[t].name, 100.0 * team_wins[t] as f32 / team_games[t].max(1) as f32, team_games[t]);
    }
    println!("higher skill (gap > 0.2) won {}/{}", better_skill_won, skill_games);
}

// ------------------------------------------------------------------ probe
// Scripted "human" on side 0 against the CPU ladder, to find exploits and
// check the difficulty curve. Based on the gameplay review's probe.

#[derive(Clone, Copy, PartialEq, Debug)]
enum Def {
    Stand,
    Wiggle,
    /// press catch exactly this many frames before contact
    Catch(i32),
    Duck,
    /// a decent human: moves, tries to catch half the balls with +-3 frames of noise
    Decent,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Off {
    Normal,
    /// frame-perfect super every time
    Super,
    Stall,
    /// supers about 40% of the time, with +-2 frames of timing noise
    Decent,
}

struct Bot {
    def: Def,
    off: Off,
    prev: Pad,
    t: u64,
    rng: Rng,
    seen_ball: u32,
    catch_at: Option<f32>,
    holder_seen: Option<usize>,
    try_super: bool,
    jump_at: Option<u64>,
    throw_at: Option<u64>,
}

impl Bot {
    fn new(def: Def, off: Off, seed: u64) -> Self {
        Bot { def, off, prev: Pad::default(), t: 0, rng: Rng(seed | 1), seen_ball: 0, catch_at: None, holder_seen: None, try_super: false, jump_at: None, throw_at: None }
    }

    fn pad(&mut self, m: &Match) -> Pad {
        self.t += 1;
        let me = m.sides[0].control;
        let p = &m.players[me];
        let b = &m.ball;
        let mut pad = Pad::default();
        let hp = |t: usize| m.players.iter().filter(|q| q.team == t).map(|q| q.hp.max(0)).sum::<i32>();
        let wiggle = |pad: &mut Pad, t: u64| {
            if (t / 30) % 2 == 0 { pad.up = true } else { pad.down = true }
            if p.y < 4.0 { pad.up = false; pad.down = true }
            if p.y > DEPTH - 4.0 { pad.down = false; pad.up = true }
        };
        let tap = |now: bool, prev: bool| now && !prev;
        match b.state {
            BallState::Held(h) if h == me => {
                if self.holder_seen != Some(h) {
                    self.holder_seen = Some(h);
                    self.try_super = match self.off {
                        Off::Super => true,
                        Off::Decent => self.rng.f() < 0.4,
                        _ => false,
                    };
                    self.jump_at = None;
                    self.throw_at = None;
                }
                let stall = self.off == Off::Stall && hp(0) > hp(1);
                if stall {
                    pad.left = p.x > 20.0;
                    wiggle(&mut pad, self.t);
                } else if p.x < 96.0 && p.z <= 0.0 && self.jump_at.is_none() {
                    pad.right = true;
                } else if self.try_super {
                    if self.jump_at.is_none() && p.z <= 0.0 && p.state == PState::Normal {
                        self.jump_at = Some(self.t);
                        let noise = if self.off == Off::Decent { self.rng.int(5) as u64 } else { 2 };
                        // apex is about 16 frames after take-off
                        self.throw_at = Some(self.t + 14 + noise);
                        pad.b = true;
                    } else if Some(self.t) == self.throw_at {
                        pad.a = true;
                    } else if self.throw_at.map(|ta| self.t > ta + 30).unwrap_or(false) {
                        pad.a = tap(true, self.prev.a);
                    }
                } else {
                    pad.a = tap(true, self.prev.a);
                }
            }
            BallState::Flying if b.live && b.team == 1 => {
                if b.id != self.seen_ball {
                    self.seen_ball = b.id;
                    self.catch_at = match self.def {
                        Def::Catch(n) => Some(n as f32),
                        Def::Decent if self.rng.f() < 0.5 => Some(4.0 + self.rng.range(-3.0, 3.0)),
                        _ => None,
                    };
                }
                let t = if b.vx.abs() > 0.1 { ((p.x - b.x).abs() - 7.0) / b.vx.abs() } else { 99.0 };
                match self.def {
                    Def::Stand => {}
                    Def::Wiggle => wiggle(&mut pad, self.t),
                    Def::Catch(_) | Def::Decent => {
                        if let Some(at) = self.catch_at {
                            if b.target == me && t <= at && t > at - 1.0 {
                                pad.a = true;
                            }
                        } else {
                            wiggle(&mut pad, self.t);
                        }
                    }
                    Def::Duck => {
                        if b.target == me && t <= 6.0 && p.state == PState::Normal {
                            pad.c = tap(true, self.prev.c);
                        }
                    }
                }
            }
            BallState::Loose => {
                if b.x < MID + 5.0 {
                    if b.x > p.x + 2.0 { pad.right = true }
                    if b.x < p.x - 2.0 { pad.left = true }
                    if b.y > p.y + 2.0 { pad.down = true }
                    if b.y < p.y - 2.0 { pad.up = true }
                }
            }
            _ => {
                if matches!(self.def, Def::Wiggle | Def::Decent) {
                    wiggle(&mut pad, self.t);
                }
                if p.x > 60.0 { pad.left = true }
            }
        }
        if let Some(w) = &m.wrench {
            if w.live && w.target == me && self.def != Def::Stand {
                pad.up = false;
                pad.down = false;
                if p.x > 40.0 { pad.left = true; pad.right = false } else { pad.right = true; pad.left = false }
            }
        }
        self.prev = pad;
        pad
    }
}

#[derive(Default)]
struct Res {
    wins: u32,
    n: u32,
    rounds: u32,
    timeouts: u32,
    hits_taken: u32,
    dead: u64,
    catches: u32,
    cpu_throws: u32,
    wrenches: u32,
}

fn probe_play(def: Def, off: Off, skill: f32, my: usize, opp: usize, seeds: u32, r: &mut Res) {
    for s in 0..seeds {
        let seed = s as u64 * 7 + 3 + my as u64 * 101 + opp as u64 * 13;
        let mut m = Match::new([my, opp], [Ctrl::Human(0), Ctrl::Cpu], [skill.max(0.55), skill], TEAMS[opp].court, seed);
        let mut bot = Bot::new(def, off, seed);
        let mut f = 0;
        let mut prev_hp = [0i32; 6];
        let mut prev_state = BallState::Loose;
        let mut had_wrench = false;
        while !m.finished() && f < 60 * 60 * 15 {
            let pad = bot.pad(&m);
            for (k, p) in m.players.iter().enumerate() {
                prev_hp[k] = p.hp;
            }
            let prev_team = m.ball.team;
            m.update(&[pad, Pad::default()]);
            m.sfx.clear();
            f += 1;
            if m.phase == Phase::Play {
                let p = &m.players[m.sides[0].control];
                if matches!(p.state, PState::Hit | PState::Down) && m.ball.state == BallState::Loose && m.ball.x < MID {
                    r.dead += 1;
                }
            }
            for (k, p) in m.players.iter().enumerate().take(3) {
                if p.hp < prev_hp[k] {
                    r.hits_taken += 1;
                }
            }
            let hw = m.wrench.as_ref().map(|w| w.live).unwrap_or(false);
            if hw && !had_wrench {
                r.wrenches += 1;
            }
            had_wrench = hw;
            if prev_state == BallState::Flying && prev_team == 1 {
                if let BallState::Held(h) = m.ball.state {
                    if h < 3 {
                        r.catches += 1;
                    }
                }
            }
            if m.ball.state == BallState::Flying && prev_state != BallState::Flying && m.ball.team == 1 && m.ball.shot != Shot::Pass {
                r.cpu_throws += 1;
            }
            prev_state = m.ball.state;
        }
        r.n += 1;
        r.rounds += m.stats.rounds;
        r.timeouts += m.stats.timeouts;
        if m.winner == Some(0) {
            r.wins += 1;
        }
    }
}

pub fn probe(per: u32) {
    println!("{:<18} {:>5} {:>5} {:>6} {:>8} {:>13} {:>9} {:>8}", "bot", "cpu", "win%", "tout%", "taken/rd", "caught/throws", "dead s/rd", "wrench/rd");
    let rows = [
        (Def::Stand, Off::Normal),
        (Def::Wiggle, Off::Normal),
        (Def::Wiggle, Off::Super),
        (Def::Duck, Off::Super),
        (Def::Catch(4), Off::Super),
        (Def::Wiggle, Off::Stall),
        (Def::Decent, Off::Decent),
    ];
    for &(def, off) in &rows {
        for &skill in &crate::app::CPU_SKILL {
            let mut r = Res::default();
            for my in 0..4 {
                for k in 1..4 {
                    probe_play(def, off, skill, my, (my + k) % 4, per, &mut r);
                }
            }
            let rd = r.rounds.max(1) as f32;
            println!(
                "{:<18} {:>5.2} {:>5.0} {:>6.0} {:>8.1} {:>6}/{:<6} {:>9.1} {:>8.2}",
                format!("{:?}/{:?}", def, off),
                skill,
                100.0 * r.wins as f32 / r.n as f32,
                100.0 * r.timeouts as f32 / rd,
                r.hits_taken as f32 / rd,
                r.catches,
                r.cpu_throws,
                r.dead as f32 / 60.0 / rd,
                r.wrenches as f32 / rd
            );
        }
    }
}
