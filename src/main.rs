use dodgeball::app::App;
use dodgeball::audio::{self, Track, ALL_SFX, ALL_TRACKS};
use dodgeball::game::{Pad, Sfx};
use dodgeball::gfx::{Frame, H, W};
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use macroquad::prelude::*;
use std::collections::HashMap;

fn conf() -> Conf {
    Conf {
        window_title: "DODGE OR DIE!".to_owned(),
        window_width: W * 3,
        window_height: H * 3,
        high_dpi: true,
        window_resizable: true,
        ..Default::default()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--shots") => dodgeball::headless::shots(args.get(2).map(String::as_str).unwrap_or("shots")),
        Some("--sim") => dodgeball::headless::sim(args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100)),
        _ => macroquad::Window::from_config(conf(), run()),
    }
}

fn pads() -> [Pad; 2] {
    let k = is_key_down;
    [
        Pad {
            up: k(KeyCode::Up),
            down: k(KeyCode::Down),
            left: k(KeyCode::Left),
            right: k(KeyCode::Right),
            a: k(KeyCode::Z),
            b: k(KeyCode::X),
            c: k(KeyCode::C),
            start: k(KeyCode::Enter),
        },
        Pad {
            up: k(KeyCode::W),
            down: k(KeyCode::S),
            left: k(KeyCode::A),
            right: k(KeyCode::D),
            a: k(KeyCode::F),
            b: k(KeyCode::G),
            c: k(KeyCode::H),
            start: k(KeyCode::Tab),
        },
    ]
}

fn sfx_volume(s: Sfx) -> f32 {
    match s {
        Sfx::Cheer => 0.5,
        Sfx::Bounce | Sfx::Pickup => 0.6,
        _ => 0.8,
    }
}

async fn run() {
    let mut sounds: HashMap<Sfx, Sound> = HashMap::new();
    for s in ALL_SFX {
        if let Ok(snd) = load_sound_from_bytes(&audio::wav(&audio::sfx_samples(s))).await {
            sounds.insert(s, snd);
        }
    }
    let mut music: HashMap<Track, Sound> = HashMap::new();
    for t in ALL_TRACKS {
        if let Ok(snd) = load_sound_from_bytes(&audio::wav(&audio::music_samples(t))).await {
            music.insert(t, snd);
        }
    }

    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64;
    let mut app = App::new(seed);
    let mut frame = Frame::new();
    let mut rgba = Vec::new();
    frame.to_rgba(&mut rgba);
    let tex = Texture2D::from_rgba8(W as u16, H as u16, &rgba);
    tex.set_filter(FilterMode::Nearest);

    let dt = 1.0 / 60.0;
    let mut acc = 0.0f32;
    let mut pending_esc = false;
    let mut scanlines = true;
    let mut playing: Option<Track> = None;
    loop {
        acc += get_frame_time().min(0.1);
        pending_esc |= is_key_pressed(KeyCode::Escape);
        if is_key_pressed(KeyCode::F1) {
            scanlines = !scanlines;
        }
        while acc >= dt {
            app.update(pads(), pending_esc);
            pending_esc = false;
            acc -= dt;
        }
        for s in app.sfx.drain(..) {
            if let Some(snd) = sounds.get(&s) {
                play_sound(snd, PlaySoundParams { looped: false, volume: sfx_volume(s) });
            }
        }
        let want = app.music();
        if want != playing {
            if let Some(snd) = playing.and_then(|p| music.get(&p)) {
                stop_sound(snd);
            }
            if let Some(snd) = want.and_then(|w| music.get(&w)) {
                play_sound(snd, PlaySoundParams { looped: true, volume: 0.45 });
            }
            playing = want;
        }
        if app.quit {
            break;
        }

        app.draw(&mut frame);
        frame.to_rgba(&mut rgba);
        tex.update_from_bytes(W as u32, H as u32, &rgba);

        clear_background(BLACK);
        let (sw, sh) = (screen_width(), screen_height());
        let scale = (sw / W as f32).min(sh / H as f32).floor().max(1.0);
        let (w, h) = (W as f32 * scale, H as f32 * scale);
        let (x, y) = (((sw - w) / 2.0).floor(), ((sh - h) / 2.0).floor());
        draw_texture_ex(&tex, x, y, WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
        if scanlines && scale >= 3.0 {
            let thick = (scale / 3.0).floor();
            for row in 0..H {
                draw_rectangle(x, y + (row as f32 + 1.0) * scale - thick, w, thick, Color::new(0.0, 0.0, 0.0, 0.22));
            }
        }
        next_frame().await;
    }
}
