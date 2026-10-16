//! One dodgeball match: 3 vs 3, best of 3 rounds. Pure simulation (no
//! drawing, no I/O), stepped at 60 Hz. The world is a court seen from the
//! side: x runs left-right, y is depth (0 = far line), z is height.

use crate::gfx::rgb;
use crate::teams::{Court, TeamDef, TEAMS};

pub const TEAM_SIZE: usize = 3;
pub const MAX_HP: i32 = 4;
pub const DEPTH: f32 = 72.0;
pub const COURT_TOP: f32 = 112.0;
pub const MID: f32 = 128.0;
pub const X_MIN: f32 = 16.0;
pub const X_MAX: f32 = 240.0;
pub const ROUND_SECS: i32 = 90;
pub const ROUNDS_TO_WIN: u32 = 2;

const JUMP_V: f32 = 3.6;
const P_GRAV: f32 = 0.22;
const CHEST: f32 = 14.0;

/// World -> screen. Far side of the court is a bit narrower (fake perspective).
pub fn proj(x: f32, y: f32, z: f32) -> (i32, i32) {
    let s = 0.8 + 0.2 * (y / DEPTH);
    ((MID + (x - MID) * s).round() as i32, (COURT_TOP + y - z).round() as i32)
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Pad {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    /// throw / catch
    pub a: bool,
    /// jump
    pub b: bool,
    /// pass
    pub c: bool,
    pub start: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 32) as u32
    }
    pub fn f(&mut self) -> f32 {
        (self.next() >> 8) as f32 / (1u32 << 24) as f32
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
    pub fn int(&mut self, n: i32) -> i32 {
        (self.next() % n.max(1) as u32) as i32
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sfx {
    Throw,
    Super,
    Hit,
    BigHit,
    Catch,
    Bounce,
    Jump,
    Whistle,
    Blip,
    Confirm,
    Ko,
    Cheer,
    Pickup,
    Pass,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shot {
    Normal,
    Dash,
    Super,
    Pass,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PState {
    Normal,
    WindUp,
    Throw,
    Catch,
    Hit,
    Down,
    Out,
    Victory,
    /// crouch: low hitbox, balls fly over
    Duck,
    /// running dive: flat and low, then slide
    Dive,
}

/// Top of a player's hitbox above their feet.
fn hit_top(s: PState) -> f32 {
    match s {
        PState::Duck => 11.0,
        PState::Dive => 9.0,
        _ => 27.0,
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Intent {
    pub mx: f32,
    pub my: f32,
    pub run: bool,
    pub throw: bool,
    pub jump: bool,
    pub pass: bool,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Ai {
    hold: i32,
    delay: i32,
    plan: u8,
    ty: f32,
    seen: u32,
    since: i32,
    react: i32,
    choice: u8,
    wander: i32,
    wx: f32,
    wy: f32,
    fetch: i32,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub team: usize,
    pub slot: usize,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub face: f32,
    pub hp: i32,
    pub state: PState,
    pub t: i32,
    pub shot: Shot,
    pub run_t: i32,
    pub anim: f32,
    pub invuln: i32,
    pub catch_cd: i32,
    pub ghost: f32,
    pub flash: i32,
    pub aim: f32,
    /// dodged the coach's wrench: next throw is a free super shot
    pub fired_up: bool,
    ai: Ai,
}

impl Player {
    pub fn alive(&self) -> bool {
        self.state != PState::Out
    }
    pub fn moving(&self) -> bool {
        self.z <= 0.0 && (self.vx.abs() > 0.2 || self.vy.abs() > 0.2)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BallState {
    Held(usize),
    Flying,
    Loose,
}

#[derive(Clone, Debug)]
pub struct Ball {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub state: BallState,
    pub shot: Shot,
    pub team: usize,
    pub target: usize,
    pub live: bool,
    pub id: u32,
    pub idle: i32,
    pub air: i32,
    pub spin: f32,
    pub trail: Vec<(f32, f32, f32)>,
}

/// The coach's wrench ("if you can dodge a wrench, you can dodge a ball").
#[derive(Clone, Debug)]
pub struct Wrench {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub team: usize,
    pub target: usize,
    pub spin: f32,
    pub live: bool,
    pub judged: bool,
    pub t: i32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Coach {
    pub cooldown: i32,
    /// frames until the wrench leaves the coach's hand
    pub warn: i32,
    /// frames left of the throw follow-through
    pub throw_t: i32,
}

pub fn coach_pos(team: usize) -> (f32, f32) {
    (if team == 0 { 56.0 } else { 200.0 }, DEPTH + 20.0)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ctrl {
    Human(usize),
    Cpu,
}

#[derive(Clone, Debug)]
pub struct Side {
    pub team: usize,
    pub ctrl: Ctrl,
    pub skill: f32,
    pub wins: u32,
    pub control: usize,
    prev: Pad,
    tap_dir: i32,
    tap_t: u64,
    run_dir: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FxKind {
    Dust,
    Star,
    Spark,
    Sweat,
    /// big starburst at the point of contact
    Impact,
}

#[derive(Clone, Debug)]
pub struct Fx {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub life: i32,
    pub max: i32,
    pub kind: FxKind,
    pub color: u32,
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub x: f32,
    pub y: f32,
    pub text: &'static str,
    pub color: u32,
    pub life: i32,
}

#[derive(Clone, Debug)]
pub struct Banner {
    pub text: String,
    pub t: i32,
    pub dur: i32,
    pub color: u32,
    /// pop in place instead of sliding in
    pub pop: bool,
    /// screen y of the text top (None = centre of the court)
    pub y: Option<i32>,
    /// logo scale (0 = pick by length)
    pub scale: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Intro,
    Play,
    RoundOver,
    MatchOver,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    pub throws: u32,
    pub hits: u32,
    pub catches: u32,
    pub supers: u32,
    pub dashes: u32,
    pub passes: u32,
    pub dodges: u32,
    pub rounds: u32,
    pub timeouts: u32,
    pub round_frames: u64,
}

pub struct Match {
    pub players: Vec<Player>,
    pub ball: Ball,
    pub sides: [Side; 2],
    pub court: Court,
    pub phase: Phase,
    pub phase_t: i32,
    pub round: u32,
    pub time: i32,
    pub frame: u64,
    pub rng: Rng,
    pub sfx: Vec<Sfx>,
    pub fx: Vec<Fx>,
    pub popups: Vec<Popup>,
    pub banner: Option<Banner>,
    pub shake: i32,
    pub hitstop: i32,
    /// frames of full-screen flash (super shot release)
    pub flash: i32,
    pub excite: f32,
    pub stats: Stats,
    pub winner: Option<usize>,
    pub wrench: Option<Wrench>,
    pub coaches: [Coach; 2],
    /// frames left to show the coach's quote, and whose coach says it
    pub quote: i32,
    pub quote_team: usize,
    ball_contact: (f32, f32, f32),
    round_t: u64,
}

fn catch_window(catch: i32) -> i32 {
    9 + (catch - 3) * 2
}

fn windup_frames(s: Shot) -> i32 {
    match s {
        Shot::Normal => 9,
        Shot::Dash => 6,
        Shot::Super => 8,
        Shot::Pass => 4,
    }
}

fn ball_gravity(s: Shot) -> f32 {
    match s {
        Shot::Normal => 0.06,
        Shot::Dash => 0.04,
        Shot::Super => 0.0,
        Shot::Pass => 0.2,
    }
}

/// Digital steering like a d-pad: -1, 0 or 1 per axis, with a dead zone.
fn steer(x: f32, y: f32, tx: f32, ty: f32, dead: f32) -> (f32, f32) {
    let ax = if tx - x > dead { 1.0 } else if x - tx > dead { -1.0 } else { 0.0 };
    let ay = if ty - y > dead { 1.0 } else if y - ty > dead { -1.0 } else { 0.0 };
    (ax, ay)
}

fn side_bounds(team: usize) -> (f32, f32) {
    if team == 0 {
        (X_MIN, MID - 6.0)
    } else {
        (MID + 6.0, X_MAX)
    }
}

pub fn toward(team: usize) -> f32 {
    if team == 0 {
        1.0
    } else {
        -1.0
    }
}

impl Match {
    pub fn new(teams: [usize; 2], ctrls: [Ctrl; 2], skills: [f32; 2], court: Court, seed: u64) -> Self {
        let side = |i: usize| Side {
            team: teams[i],
            ctrl: ctrls[i],
            skill: skills[i],
            wins: 0,
            control: i * TEAM_SIZE,
            prev: Pad::default(),
            tap_dir: 0,
            tap_t: 0,
            run_dir: 0,
        };
        let players = (0..TEAM_SIZE * 2)
            .map(|i| Player {
                team: i / TEAM_SIZE,
                slot: i % TEAM_SIZE,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                vx: 0.0,
                vy: 0.0,
                vz: 0.0,
                face: 1.0,
                hp: MAX_HP,
                state: PState::Normal,
                t: 0,
                shot: Shot::Normal,
                run_t: 0,
                anim: 0.0,
                invuln: 0,
                catch_cd: 0,
                ghost: 0.0,
                flash: 0,
                aim: 0.0,
                fired_up: false,
                ai: Ai::default(),
            })
            .collect();
        let mut m = Match {
            players,
            ball: Ball {
                x: MID,
                y: DEPTH / 2.0,
                z: 0.0,
                vx: 0.0,
                vy: 0.0,
                vz: 0.0,
                state: BallState::Loose,
                shot: Shot::Normal,
                team: 0,
                target: 0,
                live: false,
                id: 0,
                idle: 0,
                air: 0,
                spin: 0.0,
                trail: Vec::new(),
            },
            sides: [side(0), side(1)],
            court,
            phase: Phase::Intro,
            phase_t: 0,
            round: 1,
            time: ROUND_SECS * 60,
            frame: 0,
            rng: Rng(seed.max(1).wrapping_mul(0x9E3779B97F4A7C15) | 1),
            sfx: Vec::new(),
            fx: Vec::new(),
            popups: Vec::new(),
            banner: None,
            shake: 0,
            hitstop: 0,
            flash: 0,
            excite: 0.0,
            stats: Stats::default(),
            winner: None,
            wrench: None,
            coaches: [Coach::default(); 2],
            quote: 0,
            quote_team: 0,
            ball_contact: (0.0, 0.0, 0.0),
            round_t: 0,
        };
        m.reset_round();
        m
    }

    pub fn tdef(&self, team: usize) -> &'static TeamDef {
        &TEAMS[self.sides[team].team]
    }

    fn reset_round(&mut self) {
        for p in self.players.iter_mut() {
            let (lo, hi) = side_bounds(p.team);
            let k = p.slot as f32 * 14.0;
            p.x = if p.team == 0 { lo + 10.0 + k } else { hi - 10.0 - k };
            p.y = 10.0 + p.slot as f32 * 26.0;
            p.z = 0.0;
            p.vx = 0.0;
            p.vy = 0.0;
            p.vz = 0.0;
            p.face = toward(p.team);
            p.hp = MAX_HP;
            p.state = PState::Normal;
            p.t = 0;
            p.invuln = 0;
            p.catch_cd = 0;
            p.ghost = 0.0;
            p.fired_up = false;
            p.ai = Ai { fetch: 0, ..Ai::default() };
        }
        self.wrench = None;
        self.quote = 0;
        for c in self.coaches.iter_mut() {
            *c = Coach { cooldown: 300, ..Coach::default() };
        }
        let b = &mut self.ball;
        b.x = MID;
        b.y = DEPTH / 2.0;
        b.z = 30.0;
        b.vx = 0.0;
        b.vy = 0.0;
        b.vz = 0.0;
        b.state = BallState::Loose;
        b.live = false;
        b.idle = 0;
        b.trail.clear();
        self.phase = Phase::Intro;
        self.phase_t = 0;
        self.time = ROUND_SECS * 60;
        self.round_t = 0;
        for t in 0..2 {
            self.sides[t].control = t * TEAM_SIZE + 1;
        }
    }

    fn banner(&mut self, text: impl Into<String>, dur: i32, color: u32) {
        self.banner = Some(Banner { text: text.into(), t: 0, dur, color, pop: false, y: None, scale: 0 });
    }

    fn popup(&mut self, x: f32, y: f32, z: f32, text: &'static str, color: u32) {
        let (sx, sy) = proj(x, y, z);
        self.popups.push(Popup { x: sx as f32, y: sy as f32, text, color, life: 45 });
    }

    fn dust(&mut self, x: f32, y: f32, n: usize) {
        for _ in 0..n {
            let vx = self.rng.range(-0.8, 0.8);
            let vz = self.rng.range(0.2, 0.9);
            self.fx.push(Fx { x, y, z: 1.0, vx, vy: 0.0, vz, life: 18, max: 18, kind: FxKind::Dust, color: 0 });
        }
    }

    fn stars(&mut self, x: f32, y: f32, z: f32, n: usize, kind: FxKind) {
        for _ in 0..n {
            let a = self.rng.range(0.0, std::f32::consts::TAU);
            let s = self.rng.range(1.0, 2.6);
            let color = [rgb(248, 248, 120), rgb(248, 248, 248), rgb(248, 160, 48)][self.rng.int(3) as usize];
            self.fx.push(Fx { x, y, z, vx: a.cos() * s, vy: 0.0, vz: a.sin() * s + 1.0, life: 24, max: 24, kind, color });
        }
    }

    pub fn holder(&self) -> Option<usize> {
        match self.ball.state {
            BallState::Held(h) => Some(h),
            _ => None,
        }
    }

    // ------------------------------------------------------------ update

    pub fn update(&mut self, pads: &[Pad; 2]) {
        self.frame += 1;
        let mut human = [Intent::default(); 2];
        for t in 0..2 {
            if let Ctrl::Human(pi) = self.sides[t].ctrl {
                human[t] = self.human_intent(t, pads[pi]);
            }
        }
        if self.flash > 0 {
            self.flash -= 1;
        }
        if self.hitstop > 0 {
            self.hitstop -= 1;
            return;
        }
        if self.shake > 0 {
            self.shake -= 1;
        }
        self.excite *= 0.985;
        if self.quote > 0 {
            self.quote -= 1;
        }
        if self.phase != Phase::Play {
            self.wrench = None;
        }
        self.update_fx();
        self.phase_t += 1;

        match self.phase {
            Phase::Intro => {
                // round 1 opens with the five D's of dodgeball
                let chant = self.round == 1;
                let go = if chant { 128 } else { 80 };
                if self.phase_t == 1 {
                    let r = format!("ROUND {}", self.round);
                    self.banner(r, if chant { 50 } else { 70 }, rgb(248, 248, 248));
                }
                if chant {
                    let words = ["DODGE!", "DUCK!", "DIP!", "DIVE!", "...AND DODGE!"];
                    for (k, w) in words.iter().enumerate() {
                        if self.phase_t == 54 + k as i32 * 15 {
                            let last = k == words.len() - 1;
                            self.banner(*w, if last { 50 } else { 14 }, rgb(248, 216, 48));
                            if let Some(b) = self.banner.as_mut() {
                                b.pop = true;
                            }
                            self.sfx.push(if last { Sfx::Whistle } else { Sfx::Blip });
                            self.excite = 0.6;
                        }
                    }
                } else if self.phase_t == 72 {
                    self.banner("DODGE!", 50, rgb(248, 216, 48));
                    self.sfx.push(Sfx::Whistle);
                }
                if self.phase_t >= go {
                    self.phase = Phase::Play;
                    self.phase_t = 0;
                }
                self.step_world(None, &human);
            }
            Phase::Play => {
                self.round_t += 1;
                self.time -= 1;
                self.update_control();
                self.step_world(Some(()), &human);
                self.update_coaches();
                self.update_wrench();
                self.check_round_end();
            }
            Phase::RoundOver => {
                self.step_world(None, &human);
                if self.phase_t > 220 {
                    if self.sides.iter().any(|s| s.wins >= ROUNDS_TO_WIN) {
                        let w = if self.sides[0].wins >= ROUNDS_TO_WIN { 0 } else { 1 };
                        self.winner = Some(w);
                        self.phase = Phase::MatchOver;
                        self.phase_t = 0;
                        let name = self.tdef(w).short;
                        self.banner(format!("{} WIN!", name), 100000, rgb(248, 216, 48));
                        self.sfx.push(Sfx::Cheer);
                        self.excite = 1.0;
                        for p in self.players.iter_mut() {
                            if p.team == w {
                                p.state = PState::Victory;
                                p.hp = p.hp.max(1);
                                p.t = 0;
                            }
                        }
                    } else {
                        self.round += 1;
                        self.reset_round();
                    }
                }
            }
            Phase::MatchOver => {
                if self.phase_t % 80 == 0 {
                    self.excite = 1.0;
                }
                self.step_world(None, &human);
            }
        }
    }

    fn step_world(&mut self, play: Option<()>, human: &[Intent; 2]) {
        let n = self.players.len();
        let mut intents = vec![Intent::default(); n];
        if play.is_some() {
            let mut rng = self.rng;
            for i in 0..n {
                let team = self.players[i].team;
                let is_human = matches!(self.sides[team].ctrl, Ctrl::Human(_)) && self.sides[team].control == i;
                if is_human {
                    intents[i] = human[team];
                } else {
                    let (it, ai) = self.ai_intent(i, &mut rng);
                    intents[i] = it;
                    self.players[i].ai = ai;
                }
            }
            self.rng = rng;
        }
        for (i, it) in intents.iter().enumerate() {
            self.apply(i, *it);
        }
        self.update_ball();
        if play.is_some() {
            self.check_hits();
            self.check_pickup();
        }
    }

    fn update_fx(&mut self) {
        for f in self.fx.iter_mut() {
            f.x += f.vx;
            f.y += f.vy;
            f.z += f.vz;
            if f.kind == FxKind::Impact {
            } else if f.kind != FxKind::Dust {
                f.vz -= 0.12;
            } else {
                f.vx *= 0.9;
            }
            f.life -= 1;
        }
        self.fx.retain(|f| f.life > 0 && f.z > -2.0);
        for p in self.popups.iter_mut() {
            p.y -= 0.6;
            p.life -= 1;
        }
        self.popups.retain(|p| p.life > 0);
        if let Some(b) = self.banner.as_mut() {
            b.t += 1;
            if b.t > b.dur {
                self.banner = None;
            }
        }
    }

    fn human_intent(&mut self, team: usize, pad: Pad) -> Intent {
        let frame = self.frame;
        let side = &mut self.sides[team];
        let prev = side.prev;
        side.prev = pad;
        let mx = (pad.right as i32 - pad.left as i32) as f32;
        let my = (pad.down as i32 - pad.up as i32) as f32;
        for (pressed, dir) in [(pad.left && !prev.left, -1), (pad.right && !prev.right, 1)] {
            if pressed {
                if side.tap_dir == dir && frame - side.tap_t < 15 {
                    side.run_dir = dir;
                }
                side.tap_dir = dir;
                side.tap_t = frame;
            }
        }
        if side.run_dir != 0 && mx as i32 != side.run_dir {
            side.run_dir = 0;
        }
        Intent {
            mx,
            my,
            run: side.run_dir != 0,
            throw: pad.a && !prev.a,
            jump: pad.b && !prev.b,
            pass: pad.c && !prev.c,
        }
    }

    /// Which player the human steers: the ball holder, the one the ball is
    /// flying at, or else whoever is closest to the ball.
    fn update_control(&mut self) {
        for t in 0..2 {
            if !matches!(self.sides[t].ctrl, Ctrl::Human(_)) {
                continue;
            }
            let cur = self.sides[t].control;
            let b = &self.ball;
            let mine = |i: usize| self.players[i].team == t && self.players[i].alive() && self.players[i].hp > 0;
            let cand = match b.state {
                BallState::Held(h) if self.players[h].team == t => Some(h),
                BallState::Flying if mine(b.target) => Some(b.target),
                _ => {
                    let best = (t * TEAM_SIZE..(t + 1) * TEAM_SIZE).filter(|&i| mine(i)).min_by(|&a, &c| {
                        let d = |i: usize| (self.players[i].x - b.x).hypot(self.players[i].y - b.y);
                        d(a).partial_cmp(&d(c)).unwrap()
                    });
                    match best {
                        Some(i) if mine(cur) => {
                            let d = |i: usize| (self.players[i].x - b.x).hypot(self.players[i].y - b.y);
                            if d(i) + 8.0 < d(cur) {
                                Some(i)
                            } else {
                                Some(cur)
                            }
                        }
                        other => other,
                    }
                }
            };
            if let Some(c) = cand {
                let busy = matches!(self.players[cur].state, PState::WindUp | PState::Catch) && mine(cur);
                if c != cur && !busy {
                    self.sides[t].control = c;
                    self.sides[t].run_dir = 0;
                }
            }
        }
    }

    fn nearest_to_ball(&self, team: usize) -> Option<usize> {
        let b = &self.ball;
        (team * TEAM_SIZE..(team + 1) * TEAM_SIZE)
            .filter(|&i| self.players[i].state == PState::Normal && self.players[i].hp > 0)
            .min_by(|&a, &c| {
                let d = |i: usize| (self.players[i].x - b.x).abs() + (self.players[i].y - b.y).abs() * 1.4;
                d(a).partial_cmp(&d(c)).unwrap()
            })
    }

    fn ai_intent(&self, i: usize, rng: &mut Rng) -> (Intent, Ai) {
        let me = &self.players[i];
        let mut ai = me.ai;
        let mut it = Intent::default();
        if !matches!(me.state, PState::Normal | PState::WindUp | PState::Catch | PState::Throw) {
            return (it, ai);
        }
        let team = me.team;
        let side = &self.sides[team];
        let skill = side.skill;
        let mate_of_human = matches!(side.ctrl, Ctrl::Human(_));
        let td = self.tdef(team);
        let dir = toward(team);
        let b = &self.ball;
        let (lo, hi) = side_bounds(team);
        ai.since += 1;

        // 0. The coach's wrench is coming at me: get out of the way.
        if let Some(w) = &self.wrench {
            if w.live && w.target == i && !w.judged {
                if me.y < 14.0 || (me.y < DEPTH - 14.0 && (i + self.frame as usize / 90) % 2 == 0) {
                    it.my = 1.0;
                } else {
                    it.my = -1.0;
                }
                return (it, ai);
            }
        }

        // 1. A ball is coming: catch it, duck, jump it or step aside.
        if b.state == BallState::Flying && b.live && b.team != team && b.shot != Shot::Pass && b.vx.abs() > 0.1 {
            let t = (me.x - b.x) / b.vx;
            let py = if b.shot == Shot::Super && b.target == i { me.y } else { b.y + b.vy * t.max(0.0) };
            if t > -2.0 && t < 55.0 && ((py - me.y).abs() < 9.0 || b.target == i) {
                if ai.seen != b.id {
                    ai.seen = b.id;
                    ai.since = 0;
                    ai.react = 3 + (rng.f() * (1.0 - skill) * 18.0) as i32;
                    let shot_mod = match b.shot {
                        Shot::Super => 0.45,
                        Shot::Dash => 0.75,
                        _ => 1.0,
                    };
                    let catch_p = if mate_of_human {
                        0.0
                    } else {
                        (0.18 + 0.4 * skill + (td.catch - 3) as f32 * 0.08) * shot_mod
                    };
                    let r = rng.f();
                    ai.choice = if rng.f() < 0.1 + (1.0 - skill) * 0.3 {
                        0
                    } else if r < catch_p {
                        1
                    } else if r < catch_p + 0.22 {
                        2
                    } else if r < catch_p + 0.36 {
                        4
                    } else {
                        3
                    };
                }
                if ai.since >= ai.react {
                    match ai.choice {
                        1 => {
                            let win = catch_window(td.catch) as f32;
                            let lead = if b.shot == Shot::Super { win * 0.3 } else { win * 0.5 };
                            if t <= lead && me.state == PState::Normal {
                                it.throw = true;
                            }
                        }
                        2 => {
                            if t <= 11.0 && me.z <= 0.0 {
                                it.jump = true;
                            }
                        }
                        4 => {
                            // duck (pass button without the ball)
                            if t <= 8.0 && me.state == PState::Normal && me.catch_cd == 0 {
                                it.pass = true;
                            }
                        }
                        3 => {
                            let away = if me.y < 12.0 {
                                1.0
                            } else if me.y > DEPTH - 12.0 {
                                -1.0
                            } else if me.y >= py {
                                1.0
                            } else {
                                -1.0
                            };
                            it.my = away;
                        }
                        _ => {}
                    }
                }
                return (it, ai);
            }
        }

        // 2. Normal play.
        let idle = |ai: &mut Ai, rng: &mut Rng, back: bool| -> (f32, f32) {
            ai.wander -= 1;
            if ai.wander <= 0 {
                ai.wander = 30 + rng.int(70);
                let depth = if back { rng.range(0.15, 0.55) } else { rng.range(0.3, 0.85) };
                ai.wx = if team == 0 { lo + (hi - lo) * depth } else { hi - (hi - lo) * depth };
                ai.wy = (8.0 + me.slot as f32 * 28.0 + rng.range(-10.0, 10.0)).clamp(2.0, DEPTH - 2.0);
            }
            steer(me.x, me.y, ai.wx, ai.wy, 3.0)
        };

        match b.state {
            BallState::Held(h) if h == i => {
                ai.hold += 1;
                let line_x = if ai.plan == 1 { MID - dir * 80.0 } else { MID - dir * (34.0 - skill * 16.0) };
                if ai.hold < ai.delay {
                    let (mx, my) = steer(me.x, me.y, line_x, ai.ty, 3.0);
                    it.mx = mx;
                    it.my = my;
                } else {
                    match ai.plan {
                        1 => {
                            it.mx = dir;
                            it.run = true;
                            if ((MID - me.x).abs() < 26.0 && me.run_t >= 6) || ai.hold > ai.delay + 50 {
                                it.throw = true;
                            }
                        }
                        2 => {
                            if me.z <= 0.0 && me.vz == 0.0 && ai.hold < ai.delay + 3 {
                                it.jump = true;
                            } else if me.z > 0.0 && me.vz.abs() < 0.9 {
                                it.throw = true;
                            } else if me.z <= 0.0 && ai.hold > ai.delay + 8 {
                                it.throw = true;
                            }
                        }
                        3 => it.pass = true,
                        _ => it.throw = true,
                    }
                }
            }
            BallState::Held(h) if self.players[h].team == team => {
                let (mx, my) = idle(&mut ai, rng, false);
                it.mx = mx;
                it.my = my;
            }
            BallState::Held(_) => {
                let (mx, my) = idle(&mut ai, rng, true);
                it.mx = mx;
                it.my = my;
            }
            BallState::Loose => {
                let reachable = if team == 0 { b.x < MID + 5.0 } else { b.x > MID - 5.0 };
                if reachable && !mate_of_human && self.nearest_to_ball(team) == Some(i) {
                    ai.fetch += 1;
                    if ai.fetch > (4.0 + (1.0 - skill) * 14.0) as i32 {
                        let (mx, my) = steer(me.x, me.y, b.x, b.y, 1.5);
                        it.mx = mx;
                        it.my = my;
                        it.run = (b.x - me.x).abs() > 28.0;
                    }
                } else {
                    ai.fetch = 0;
                    let (mx, my) = idle(&mut ai, rng, false);
                    it.mx = mx;
                    it.my = my;
                }
            }
            BallState::Flying => {
                let (mx, my) = idle(&mut ai, rng, false);
                it.mx = mx;
                it.my = my;
            }
        }
        (it, ai)
    }

    /// Called when a player gets the ball: the CPU decides how to throw it.
    fn plan_throw(&mut self, i: usize, allow_pass: bool) {
        let team = self.players[i].team;
        let skill = self.sides[team].skill;
        let mut rng = self.rng;
        let r = rng.f();
        let mates = (team * TEAM_SIZE..(team + 1) * TEAM_SIZE).filter(|&j| j != i && self.players[j].hp > 0).count();
        let plan = if r < 0.1 + 0.3 * skill {
            2
        } else if r < 0.32 + 0.35 * skill {
            1
        } else if allow_pass && mates > 0 && r < 0.4 + 0.35 * skill {
            3
        } else {
            0
        };
        let ty = self.choose_target(i, 0.0, false, &mut rng).map(|t| self.players[t].y).unwrap_or(DEPTH / 2.0);
        let p = &mut self.players[i];
        p.ai.hold = 0;
        p.ai.delay = 10 + (rng.f() * 40.0 * (1.3 - skill)) as i32;
        p.ai.plan = plan;
        p.ai.ty = (ty + rng.range(-8.0, 8.0)).clamp(0.0, DEPTH);
        self.rng = rng;
    }

    fn apply(&mut self, i: usize, it: Intent) {
        let team = self.players[i].team;
        let td = self.tdef(team);
        let walk = 1.1 + (td.speed - 3) as f32 * 0.12;
        let run = 2.1 + (td.speed - 3) as f32 * 0.2;
        let win = catch_window(td.catch);
        let holding = self.ball.state == BallState::Held(i);
        let dir = toward(team);
        let mut jumped = false;
        let mut release = false;
        let mut want_pass = false;
        let mut landed_hard = false;
        let mut went_down = false;
        let mut ko = false;
        let mut dove = false;
        let mut slide_dust = false;
        let p = &mut self.players[i];
        p.t += 1;
        if p.invuln > 0 {
            p.invuln -= 1;
        }
        if p.catch_cd > 0 {
            p.catch_cd -= 1;
        }
        if p.flash > 0 {
            p.flash -= 1;
        }
        let ground = p.z <= 0.0;
        match p.state {
            PState::Normal => {
                p.aim = it.my;
                if ground {
                    let running = it.run && it.mx != 0.0;
                    let spd = if running { run } else { walk };
                    p.vx = it.mx * spd;
                    p.vy = it.my * walk * 0.75;
                    let was_running = p.run_t >= 6;
                    p.run_t = if running { p.run_t + 1 } else { 0 };
                    if it.jump && was_running && !holding {
                        // running dive: low and long
                        p.state = PState::Dive;
                        p.t = 0;
                        p.vx = p.vx.signum() * (run + 1.0);
                        p.vy = it.my * 0.8;
                        p.vz = 1.8;
                        p.face = p.vx.signum();
                        dove = true;
                    } else if it.jump {
                        p.vz = JUMP_V;
                        jumped = true;
                    }
                }
                if p.state == PState::Dive {
                    // committed to the dive this frame
                } else
                if it.throw {
                    if holding {
                        p.shot = if p.fired_up {
                            p.fired_up = false;
                            Shot::Super
                        } else if !ground && p.vz.abs() < 1.2 {
                            Shot::Super
                        } else if p.run_t >= 6 {
                            Shot::Dash
                        } else {
                            Shot::Normal
                        };
                        p.state = PState::WindUp;
                        p.t = 0;
                        if p.shot == Shot::Super {
                            p.vz = 0.0;
                        }
                    } else if ground && p.catch_cd == 0 {
                        p.state = PState::Catch;
                        p.t = 0;
                        p.vx = 0.0;
                        p.vy = 0.0;
                    }
                } else if it.pass && holding {
                    want_pass = true;
                } else if it.pass && ground && p.catch_cd == 0 {
                    p.state = PState::Duck;
                    p.t = 0;
                    p.vx = 0.0;
                    p.vy = 0.0;
                }
                if p.state == PState::Normal {
                    let away = p.vx * dir < -0.1;
                    p.face = if away && !holding && ground { -dir } else { dir };
                }
                p.anim += (p.vx.abs() + p.vy.abs()).min(2.5) * 0.12;
            }
            PState::WindUp => {
                p.face = dir;
                if ground {
                    if p.shot == Shot::Dash {
                        p.vx *= 0.93;
                    } else {
                        p.vx *= 0.6;
                        p.vy *= 0.6;
                    }
                }
                if p.t >= windup_frames(p.shot) {
                    release = true;
                }
            }
            PState::Throw => {
                if ground {
                    p.vx *= 0.8;
                    p.vy *= 0.8;
                }
                if p.t >= 14 {
                    p.state = PState::Normal;
                }
            }
            PState::Catch => {
                p.vx *= 0.8;
                if p.t >= win + 12 {
                    p.state = PState::Normal;
                    p.catch_cd = 14;
                }
            }
            PState::Hit => {
                if ground && p.t > 3 {
                    p.state = PState::Down;
                    p.t = 0;
                    p.vx *= 0.3;
                    went_down = true;
                }
            }
            PState::Down => {
                p.vx *= 0.85;
                p.vy = 0.0;
                if p.t >= 50 {
                    if p.hp <= 0 {
                        p.state = PState::Out;
                        p.ghost = 0.0;
                        ko = true;
                    } else {
                        p.state = PState::Normal;
                        p.invuln = 45;
                    }
                    p.t = 0;
                }
            }
            PState::Duck => {
                p.vx = 0.0;
                p.vy = 0.0;
                if p.t >= 22 {
                    p.state = PState::Normal;
                    p.catch_cd = 26;
                }
            }
            PState::Dive => {
                if ground && p.t > 4 {
                    p.vx *= 0.86;
                    p.vy *= 0.86;
                    if p.t % 4 == 0 && p.vx.abs() > 0.6 {
                        slide_dust = true;
                    }
                }
                if p.t >= 40 {
                    p.state = PState::Normal;
                    p.catch_cd = 10;
                    p.face = dir;
                }
            }
            PState::Out => {
                p.ghost += 0.7;
                p.vx = 0.0;
                p.vy = 0.0;
            }
            PState::Victory => {
                p.vx = 0.0;
                p.vy = 0.0;
                p.face = dir;
                if ground && p.t % 34 == 0 {
                    p.vz = 2.6;
                }
            }
        }
        // physics
        let hang = p.state == PState::WindUp && p.shot == Shot::Super;
        if !hang && p.state != PState::Out {
            p.vz -= P_GRAV;
        }
        p.z += p.vz;
        if p.z <= 0.0 {
            if p.vz < -2.8 && p.state != PState::Hit {
                landed_hard = true;
            }
            p.z = 0.0;
            p.vz = 0.0;
        }
        p.x += p.vx;
        p.y += p.vy;
        let (lo, hi) = side_bounds(team);
        p.x = p.x.clamp(lo, hi);
        p.y = p.y.clamp(0.0, DEPTH);
        let (px, py) = (p.x, p.y);

        if jumped {
            self.sfx.push(Sfx::Jump);
            self.dust(px, py, 3);
        }
        if dove {
            self.sfx.push(Sfx::Pass);
            self.dust(px, py, 4);
        }
        if slide_dust {
            self.dust(px, py, 1);
        }
        if landed_hard {
            self.dust(px, py, 4);
        }
        if went_down {
            self.sfx.push(Sfx::Bounce);
            self.dust(px, py, 8);
            self.shake = self.shake.max(3);
        }
        if ko {
            self.sfx.push(Sfx::Ko);
            self.popup(px, py, 30.0, "K.O.!", rgb(248, 72, 72));
            self.excite = 1.0;
        }
        if release {
            self.release_throw(i);
        }
        if want_pass {
            self.pass(i);
        }
    }

    fn choose_target(&self, i: usize, aim: f32, human: bool, rng: &mut Rng) -> Option<usize> {
        let me = &self.players[i];
        let opp = 1 - me.team;
        let mut c: Vec<usize> = (opp * TEAM_SIZE..(opp + 1) * TEAM_SIZE)
            .filter(|&j| matches!(self.players[j].state, PState::Normal | PState::WindUp | PState::Throw | PState::Catch | PState::Duck | PState::Dive))
            .collect();
        if c.is_empty() {
            c = (opp * TEAM_SIZE..(opp + 1) * TEAM_SIZE).filter(|&j| self.players[j].hp > 0).collect();
        }
        if c.is_empty() {
            return None;
        }
        if human {
            c.sort_by(|&a, &b| self.players[a].y.partial_cmp(&self.players[b].y).unwrap());
            let base = (0..c.len())
                .min_by(|&a, &b| {
                    let d = |k: usize| (self.players[c[k]].y - me.y).abs();
                    d(a).partial_cmp(&d(b)).unwrap()
                })
                .unwrap() as i32;
            let k = (base + aim as i32).clamp(0, c.len() as i32 - 1);
            Some(c[k as usize])
        } else {
            c.into_iter()
                .map(|j| {
                    let q = &self.players[j];
                    let s = (q.y - me.y).abs() + (q.x - me.x).abs() * 0.15 + q.hp as f32 * 5.0 - rng.f() * 24.0;
                    (j, s)
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(j, _)| j)
        }
    }

    fn release_throw(&mut self, i: usize) {
        let team = self.players[i].team;
        let human = matches!(self.sides[team].ctrl, Ctrl::Human(_)) && self.sides[team].control == i;
        let mut rng = self.rng;
        let target = self.choose_target(i, self.players[i].aim, human, &mut rng);
        self.rng = rng;
        let shot = self.players[i].shot;
        let power = self.tdef(team).power;
        let p = &self.players[i];
        let (hx, hy, hz) = (p.x + p.face * 7.0, p.y, p.z + 18.0);
        let (tx, ty, tz) = match target {
            Some(t) => {
                let q = &self.players[t];
                (q.x, q.y, if matches!(q.state, PState::Duck | PState::Dive) { 5.0 } else { CHEST })
            }
            None => (hx + p.face * 60.0, hy, CHEST),
        };
        let speed = match shot {
            Shot::Normal => 3.4,
            Shot::Dash => 4.6,
            _ => 5.6,
        } + (power - 3) as f32 * 0.3;
        let (dx, dy) = (tx - hx, ty - hy);
        let d = dx.hypot(dy).max(1.0);
        let g = ball_gravity(shot);
        let t = d / speed;
        let b = &mut self.ball;
        b.x = hx;
        b.y = hy;
        b.z = hz;
        b.vx = dx / d * speed;
        b.vy = dy / d * speed;
        b.vz = (tz - hz + 0.5 * g * t * t) / t;
        b.state = BallState::Flying;
        b.shot = shot;
        b.team = team;
        b.target = target.unwrap_or(0);
        b.live = true;
        b.id += 1;
        b.air = 0;
        b.trail.clear();
        let p = &mut self.players[i];
        p.state = PState::Throw;
        p.t = 0;
        p.run_t = 0;
        self.stats.throws += 1;
        match shot {
            Shot::Super => {
                self.stats.supers += 1;
                self.sfx.push(Sfx::Super);
                // up in the sky, so the shot itself stays in view
                self.banner("SUPER SHOT!", 40, rgb(248, 120, 32));
                if let Some(b) = self.banner.as_mut() {
                    b.y = Some(40);
                    b.scale = 2;
                }
                self.flash = 2;
                self.excite = (self.excite + 0.6).min(1.0);
                self.shake = self.shake.max(4);
            }
            Shot::Dash => {
                self.stats.dashes += 1;
                self.sfx.push(Sfx::Throw);
                self.sfx.push(Sfx::Pass);
            }
            _ => self.sfx.push(Sfx::Throw),
        }
    }

    fn pass(&mut self, i: usize) {
        let team = self.players[i].team;
        let me = &self.players[i];
        let mate = (team * TEAM_SIZE..(team + 1) * TEAM_SIZE)
            .filter(|&j| j != i && self.players[j].state == PState::Normal && self.players[j].hp > 0)
            .min_by(|&a, &b| {
                let d = |j: usize| (self.players[j].x - MID).abs();
                d(a).partial_cmp(&d(b)).unwrap()
            });
        let Some(j) = mate else { return };
        let q = &self.players[j];
        let (hx, hy, hz) = (me.x + me.face * 6.0, me.y, me.z + 18.0);
        let (dx, dy) = (q.x - hx, q.y - hy);
        let d = dx.hypot(dy).max(1.0);
        let speed = 3.0;
        let g = ball_gravity(Shot::Pass);
        let t = d / speed;
        let b = &mut self.ball;
        b.x = hx;
        b.y = hy;
        b.z = hz;
        b.vx = dx / d * speed;
        b.vy = dy / d * speed;
        b.vz = (18.0 - hz + 0.5 * g * t * t) / t;
        b.state = BallState::Flying;
        b.shot = Shot::Pass;
        b.team = team;
        b.target = j;
        b.live = false;
        b.id += 1;
        b.air = 0;
        b.trail.clear();
        let p = &mut self.players[i];
        p.state = PState::Throw;
        p.t = 6;
        self.stats.passes += 1;
        self.sfx.push(Sfx::Pass);
    }

    fn update_ball(&mut self) {
        let mut bounce = false;
        let mut pass_caught: Option<usize> = None;
        match self.ball.state {
            BallState::Held(h) => {
                let p = &self.players[h];
                let b = &mut self.ball;
                b.x = p.x + p.face * 7.0;
                b.y = p.y + 0.5;
                b.z = p.z + 16.0;
                b.vx = 0.0;
                b.vy = 0.0;
                b.vz = 0.0;
                b.idle = 0;
            }
            BallState::Flying => {
                let tgt = self.ball.target;
                let (tx, ty, talive, tlow) = {
                    let q = &self.players[tgt];
                    (
                        q.x,
                        q.y,
                        matches!(q.state, PState::Normal | PState::WindUp | PState::Throw | PState::Catch | PState::Duck | PState::Dive),
                        matches!(q.state, PState::Duck | PState::Dive),
                    )
                };
                let b = &mut self.ball;
                b.air += 1;
                if b.shot == Shot::Super {
                    if talive && (tx - b.x) * b.vx > 0.0 {
                        b.vy = (b.vy + ((ty - b.y) * 0.06).clamp(-0.22, 0.22)).clamp(-2.5, 2.5);
                    }
                    // a super shot dives after a ducking target
                    let tz = if tlow { 5.0 } else { CHEST };
                    b.vz = (tz - b.z) * 0.12;
                } else {
                    b.vz -= ball_gravity(b.shot);
                }
                b.x += b.vx;
                b.y += b.vy;
                b.z += b.vz;
                b.spin += b.vx * 0.3;
                b.trail.push((b.x, b.y, b.z));
                if b.trail.len() > 14 {
                    b.trail.remove(0);
                }
                if b.shot == Shot::Pass {
                    let q = &self.players[tgt];
                    if (b.x - q.x).abs() < 9.0 && (b.y - q.y).abs() < 7.0 && b.z < q.z + 30.0 && b.vz < 0.0 && q.state == PState::Normal
                    {
                        pass_caught = Some(tgt);
                    }
                }
                if pass_caught.is_none() {
                    if b.z <= 0.0 {
                        b.z = 0.0;
                        b.vz = (-b.vz * 0.5).max(1.5);
                        b.vx *= 0.6;
                        b.vy *= 0.6;
                        b.state = BallState::Loose;
                        b.live = false;
                        bounce = true;
                    } else if b.x < 10.0 || b.x > 246.0 {
                        b.x = b.x.clamp(10.0, 246.0);
                        b.vx = -b.vx * 0.35;
                        b.state = BallState::Loose;
                        b.live = false;
                        bounce = true;
                    } else if b.y < -6.0 || b.y > DEPTH + 6.0 || b.air > 240 {
                        b.y = b.y.clamp(-6.0, DEPTH + 6.0);
                        b.vy = -b.vy * 0.5;
                        b.state = BallState::Loose;
                        b.live = false;
                    }
                }
            }
            BallState::Loose => {
                let b = &mut self.ball;
                b.vz -= 0.25;
                b.x += b.vx;
                b.y += b.vy;
                b.z += b.vz;
                b.spin += b.vx * 0.3;
                if !b.trail.is_empty() {
                    b.trail.remove(0);
                }
                if b.z <= 0.0 {
                    b.z = 0.0;
                    if b.vz < -1.2 {
                        b.vz = -b.vz * 0.55;
                        b.vx *= 0.8;
                        b.vy *= 0.8;
                        bounce = b.vz > 0.9;
                    } else {
                        b.vz = 0.0;
                        b.vx *= 0.94;
                        b.vy *= 0.94;
                    }
                }
                if b.x < 10.0 || b.x > 246.0 {
                    b.x = b.x.clamp(10.0, 246.0);
                    b.vx = -b.vx * 0.5;
                }
                if b.y < -6.0 || b.y > DEPTH + 6.0 {
                    b.y = b.y.clamp(-6.0, DEPTH + 6.0);
                    b.vy = -b.vy * 0.5;
                }
                b.idle += 1;
            }
        }
        if bounce {
            self.sfx.push(Sfx::Bounce);
        }
        if let Some(j) = pass_caught {
            self.ball.state = BallState::Held(j);
            self.sfx.push(Sfx::Pickup);
            self.plan_throw(j, false);
        }
        // A ball nobody fetches (should not happen): hand it to the side it rests on.
        if self.ball.state == BallState::Loose && self.ball.idle > 600 && self.phase == Phase::Play {
            let team = if self.ball.x < MID { 0 } else { 1 };
            if let Some(j) = self.nearest_to_ball(team) {
                self.ball.state = BallState::Held(j);
                self.plan_throw(j, true);
            }
        }
    }

    fn check_hits(&mut self) {
        let b = self.ball.clone();
        if b.state != BallState::Flying || !b.live || b.shot == Shot::Pass {
            return;
        }
        for j in 0..self.players.len() {
            let p = &self.players[j];
            if p.team == b.team || matches!(p.state, PState::Out | PState::Down | PState::Hit) || p.invuln > 0 {
                continue;
            }
            if (b.x - p.x).abs() < 7.0 && (b.y - p.y).abs() < 6.5 && b.z > p.z - 2.0 && b.z < p.z + hit_top(p.state) {
                let mut win = catch_window(self.tdef(p.team).catch);
                if b.shot == Shot::Super {
                    win /= 2;
                }
                let facing = p.face * b.vx < 0.0;
                if p.state == PState::Catch && facing && p.t <= win {
                    self.catch(j);
                } else {
                    self.hit(j);
                }
                return;
            }
        }
        // A clean dodge: the ball just passed its target without contact.
        // Name it after the five D's of dodgeball.
        let q = &self.players[b.target];
        if q.team != b.team && (q.x - b.x) * b.vx < -10.0 && (q.x - (b.x - b.vx)) * b.vx >= -10.0 {
            self.stats.dodges += 1;
            if (q.y - b.y).abs() < 14.0 && q.alive() && q.hp > 0 {
                let word = match q.state {
                    PState::Duck => "DUCK!",
                    PState::Dive => "DIVE!",
                    _ if q.z > 0.0 => "DIP!",
                    _ => "DODGE!",
                };
                let (x, y, z) = (q.x, q.y, q.z);
                self.popup(x, y, z + 34.0, word, rgb(96, 224, 248));
            }
        }
    }

    fn behind(&self, t: usize) -> bool {
        let hp = |team: usize| self.players.iter().filter(|p| p.team == team).map(|p| p.hp.max(0)).sum::<i32>();
        let o = 1 - t;
        hp(t) + 2 < hp(o) || self.sides[t].wins < self.sides[o].wins
    }

    /// When a human's crew falls behind, their coach throws a wrench at them.
    /// Dodge it and the next throw is a free super shot.
    fn update_coaches(&mut self) {
        for t in 0..2 {
            if !matches!(self.sides[t].ctrl, Ctrl::Human(_)) {
                continue;
            }
            let behind = self.behind(t);
            let target = self.sides[t].control;
            let ball_incoming = self.ball.state == BallState::Flying && self.ball.live && self.ball.team != t;
            let ready = self.players[target].state == PState::Normal && !self.players[target].fired_up;
            let c = &mut self.coaches[t];
            if c.cooldown > 0 {
                c.cooldown -= 1;
            }
            if c.throw_t > 0 {
                c.throw_t -= 1;
            }
            if c.warn > 0 {
                c.warn -= 1;
                if c.warn == 0 {
                    c.throw_t = 16;
                    self.throw_wrench(t, target);
                }
            } else if self.wrench.is_none() && c.cooldown == 0 && behind && ready && !ball_incoming {
                c.warn = 45;
                c.cooldown = 780 + self.rng.int(300);
                self.quote = 170;
                self.quote_team = t;
                let (cx, cy) = coach_pos(t);
                self.popup(cx, cy, 40.0, "HEY!", rgb(248, 248, 248));
                self.sfx.push(Sfx::Whistle);
            }
        }
    }

    fn throw_wrench(&mut self, team: usize, target: usize) {
        let (cx, cy) = coach_pos(team);
        let (hz, speed, g) = (22.0, 3.0, 0.08);
        let q = &self.players[target];
        let (dx, dy) = (q.x - cx, q.y - cy);
        let d = dx.hypot(dy).max(1.0);
        let t = d / speed;
        self.wrench = Some(Wrench {
            x: cx,
            y: cy,
            z: hz,
            vx: dx / d * speed,
            vy: dy / d * speed,
            vz: (CHEST - hz + 0.5 * g * t * t) / t,
            team,
            target,
            spin: 0.0,
            live: true,
            judged: false,
            t: 0,
        });
        self.sfx.push(Sfx::Throw);
    }

    fn update_wrench(&mut self) {
        let Some(mut w) = self.wrench.take() else { return };
        w.t += 1;
        w.vz -= 0.08;
        w.x += w.vx;
        w.y += w.vy;
        w.z += w.vz;
        if w.live {
            w.spin += 0.45;
        }
        if w.live {
            let hit = (0..self.players.len()).find(|&j| {
                let p = &self.players[j];
                p.team == w.team
                    && !matches!(p.state, PState::Out | PState::Down | PState::Hit)
                    && p.invuln == 0
                    && (w.x - p.x).abs() < 7.0
                    && (w.y - p.y).abs() < 6.5
                    && w.z > p.z - 2.0
                    && w.z < p.z + hit_top(p.state)
            });
            if let Some(j) = hit {
                // a wrench hurts your pride, not your hearts
                let push = w.vx.signum();
                let p = &mut self.players[j];
                p.state = PState::Hit;
                p.t = 0;
                p.vx = push * 1.2;
                p.vz = 2.2;
                p.flash = 6;
                let (x, y, z) = (p.x, p.y, p.z);
                w.live = false;
                w.vx *= -0.3;
                w.vy *= 0.3;
                w.vz = 2.0;
                self.sfx.push(Sfx::Hit);
                self.popup(x, y, z + 32.0, "OUCH!", rgb(248, 72, 72));
                self.stars(x, y, z + 20.0, 5, FxKind::Star);
                self.hitstop = 4;
                self.shake = 4;
            } else if !w.judged {
                let q = &self.players[w.target];
                let ahead = (q.x - w.x) * w.vx + (q.y - w.y) * w.vy;
                if ahead < -40.0 {
                    w.judged = true;
                    if q.alive() && q.hp > 0 && !matches!(q.state, PState::Hit | PState::Down) {
                        let (x, y, z) = (q.x, q.y, q.z);
                        self.players[w.target].fired_up = true;
                        self.popup(x, y, z + 34.0, "FIRED UP!", rgb(248, 136, 32));
                        self.sfx.push(Sfx::Confirm);
                        self.excite = 1.0;
                    }
                }
            }
        }
        if w.z <= 0.0 {
            w.z = 0.0;
            if w.live {
                w.live = false;
                self.sfx.push(Sfx::Bounce);
            }
            w.vz = -w.vz * 0.4;
            w.vx *= 0.7;
            w.vy *= 0.7;
        }
        let gone = w.t > 160 || w.y < -30.0 || w.x < -10.0 || w.x > 266.0;
        if !gone {
            self.wrench = Some(w);
        }
    }

    fn catch(&mut self, j: usize) {
        let shot = self.ball.shot;
        let b = &mut self.ball;
        b.state = BallState::Held(j);
        b.live = false;
        let p = &mut self.players[j];
        p.state = PState::Normal;
        p.t = 0;
        p.vx = -p.face * if shot == Shot::Super { 3.0 } else { 1.2 };
        let (x, y, z) = (p.x, p.y, p.z);
        self.sfx.push(Sfx::Catch);
        self.popup(x, y, z + 32.0, "CATCH!", rgb(120, 248, 120));
        self.stars(x, y, z + 16.0, 5, FxKind::Spark);
        self.excite = (self.excite + 0.4).min(1.0);
        self.stats.catches += 1;
        self.hitstop = 4;
        self.plan_throw(j, true);
    }

    fn hit(&mut self, j: usize) {
        self.ball_contact = (self.ball.x, self.ball.y, self.ball.z);
        let shot = self.ball.shot;
        let big = shot == Shot::Super;
        let dmg = if big { 2 } else { 1 };
        let bvx = self.ball.vx;
        let bvy = self.ball.vy;
        let push = bvx.signum();
        let jitter = self.rng.range(-0.6, 0.6);
        let p = &mut self.players[j];
        p.hp -= dmg;
        p.state = PState::Hit;
        p.t = 0;
        p.vx = push * if big { 2.6 } else { 1.4 };
        p.vy = 0.0;
        p.vz = if big { 3.8 } else { 2.4 };
        p.face = -push;
        p.flash = 6;
        let (x, y, z) = (p.x, p.y, p.z);
        let b = &mut self.ball;
        b.state = BallState::Loose;
        b.live = false;
        b.vx = -bvx * 0.22;
        b.vy = bvy * 0.3 + jitter;
        b.vz = 2.6;
        self.hitstop = if big { 10 } else { 5 };
        self.shake = if big { 10 } else { 5 };
        self.sfx.push(if big { Sfx::BigHit } else { Sfx::Hit });
        self.popup(x, y, z + 30.0, if big { "WHAM!" } else { "POW!" }, rgb(248, 232, 64));
        self.stars(x, y, z + 14.0, if big { 10 } else { 6 }, FxKind::Star);
        self.fx.push(Fx { x, y, z: z + 26.0, vx: -push * 0.5, vy: 0.0, vz: 1.2, life: 30, max: 30, kind: FxKind::Sweat, color: 0 });
        let (bx, by, bz) = (self.ball_contact.0, self.ball_contact.1, self.ball_contact.2);
        self.fx.push(Fx { x: bx, y: by + 0.5, z: bz, vx: 0.0, vy: 0.0, vz: 0.0, life: 7, max: 7, kind: FxKind::Impact, color: 0 });
        self.excite = (self.excite + 0.5).min(1.0);
        self.stats.hits += 1;
    }

    fn check_pickup(&mut self) {
        let b = &self.ball;
        if b.state != BallState::Loose || b.z > 12.0 {
            return;
        }
        let best = (0..self.players.len())
            .filter(|&j| {
                let p = &self.players[j];
                p.state == PState::Normal && p.hp > 0 && (p.x - b.x).abs() < 9.0 && (p.y - b.y).abs() < 7.0
            })
            .min_by(|&a, &c| {
                let d = |j: usize| (self.players[j].x - b.x).abs() + (self.players[j].y - b.y).abs();
                d(a).partial_cmp(&d(c)).unwrap()
            });
        if let Some(j) = best {
            self.ball.state = BallState::Held(j);
            self.ball.trail.clear();
            self.sfx.push(Sfx::Pickup);
            self.plan_throw(j, true);
        }
    }

    fn check_round_end(&mut self) {
        let mut out = [true; 2];
        let mut hp = [0; 2];
        for p in &self.players {
            if !(p.state == PState::Out && p.ghost > 20.0) {
                out[p.team] = false;
            }
            hp[p.team] += p.hp.max(0);
        }
        let winner = if out[0] {
            Some(1)
        } else if out[1] {
            Some(0)
        } else if self.time <= 0 {
            self.stats.timeouts += 1;
            match hp[0].cmp(&hp[1]) {
                std::cmp::Ordering::Greater => Some(0),
                std::cmp::Ordering::Less => Some(1),
                std::cmp::Ordering::Equal => None,
            }
        } else {
            return;
        };
        self.stats.rounds += 1;
        self.stats.round_frames += self.round_t;
        self.phase = Phase::RoundOver;
        self.phase_t = 0;
        self.sfx.push(Sfx::Whistle);
        self.sfx.push(Sfx::Cheer);
        self.excite = 1.0;
        let timeout = self.time <= 0;
        match winner {
            Some(w) => {
                self.sides[w].wins += 1;
                let text = if timeout { "TIME UP!".to_string() } else { format!("{} TAKE IT!", self.tdef(w).short) };
                self.banner(text, 200, rgb(248, 248, 248));
                for p in self.players.iter_mut() {
                    if p.team == w && p.alive() && matches!(p.state, PState::Normal | PState::Throw | PState::Catch) {
                        p.state = PState::Victory;
                        p.t = 0;
                    }
                }
            }
            None => self.banner("DRAW!", 200, rgb(248, 248, 248)),
        }
    }

    pub fn finished(&self) -> bool {
        self.phase == Phase::MatchOver
    }
}
