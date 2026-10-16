use dodgeball::app::{App, Scene};
use dodgeball::game::{Ctrl, Match, Pad};
use dodgeball::gfx::Frame;
use dodgeball::teams::{Court, TEAMS};

#[test]
fn cpu_matches_always_finish() {
    for seed in 1..=12u64 {
        let a = (seed % 4) as usize;
        let b = ((seed + 1 + seed % 3) % 4) as usize;
        let b = if a == b { (b + 1) % 4 } else { b };
        let mut m = Match::new([a, b], [Ctrl::Cpu, Ctrl::Cpu], [0.5, 0.7], TEAMS[b].court, seed);
        let mut frames = 0;
        while !m.finished() {
            m.update(&[Pad::default(); 2]);
            frames += 1;
            assert!(frames < 60 * 60 * 15, "match {seed} did not finish");
        }
        assert!(m.winner.is_some());
        assert!(m.stats.throws > 0 && m.stats.hits > 0);
    }
}

#[test]
fn idle_human_loses_but_match_finishes() {
    let mut m = Match::new([0, 1], [Ctrl::Human(0), Ctrl::Cpu], [0.5, 0.6], Court::Beach, 9);
    let mut frames = 0;
    while !m.finished() {
        m.update(&[Pad::default(); 2]);
        frames += 1;
        assert!(frames < 60 * 60 * 15);
    }
    assert_eq!(m.winner, Some(1));
}

#[test]
fn every_scene_draws() {
    let mut app = App::new(3);
    let mut f = Frame::new();
    for scene in [Scene::Title, Scene::Select, Scene::Versus, Scene::Continue, Scene::GameOver, Scene::Champion] {
        app.scene = scene;
        app.ladder = vec![1, 2, 3];
        app.draw(&mut f);
    }
    app.start_demo();
    for _ in 0..600 {
        app.update([Pad::default(); 2], false);
        app.draw(&mut f);
    }
}

#[test]
fn sounds_render() {
    for s in dodgeball::audio::ALL_SFX {
        assert!(!dodgeball::audio::sfx_samples(s).is_empty());
    }
    for t in dodgeball::audio::ALL_TRACKS {
        let wav = dodgeball::audio::wav(&dodgeball::audio::music_samples(t));
        assert!(wav.len() > 44 + 44100);
    }
}
