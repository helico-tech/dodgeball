//! The four crews, their players and their home courts.

use crate::gfx::rgb;
use crate::sprites::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Court {
    Beach,
    Boardwalk,
    Slopes,
    Skatepark,
}

impl Court {
    pub fn name(self) -> &'static str {
        match self {
            Court::Beach => "MALIBU BEACH",
            Court::Boardwalk => "VENICE BOARDWALK",
            Court::Slopes => "POWDER PEAK",
            Court::Skatepark => "DOGTOWN SKATEPARK",
        }
    }
}

pub struct PlayerDef {
    pub name: &'static str,
    pub style: usize,
    pub skin: (u32, u32),
    pub hair: (u32, u32),
}

pub struct TeamDef {
    pub name: &'static str,
    pub short: &'static str,
    pub shirt: (u32, u32),
    pub pants: (u32, u32),
    pub acc: (u32, u32),
    /// power, speed, catch: 1..=5
    pub power: i32,
    pub speed: i32,
    pub catch: i32,
    pub court: Court,
    pub players: [PlayerDef; 3],
}

const SKIN_LIGHT: (u32, u32) = (rgb(248, 200, 160), rgb(216, 144, 104));
const SKIN_TAN: (u32, u32) = (rgb(224, 160, 104), rgb(176, 112, 64));
const SKIN_DARK: (u32, u32) = (rgb(160, 96, 56), rgb(112, 64, 32));

const BLOND: (u32, u32) = (rgb(248, 216, 72), rgb(200, 152, 32));
const BROWN: (u32, u32) = (rgb(136, 80, 32), rgb(88, 48, 16));
const BLACK: (u32, u32) = (rgb(56, 48, 72), rgb(24, 24, 40));
const RED: (u32, u32) = (rgb(232, 88, 32), rgb(160, 48, 16));
const PINK: (u32, u32) = (rgb(248, 104, 184), rgb(184, 48, 128));
const LIME: (u32, u32) = (rgb(144, 248, 64), rgb(64, 168, 32));

pub const TEAMS: [TeamDef; 4] = [
    TeamDef {
        name: "MALIBU MAULERS",
        short: "MALIBU",
        shirt: (rgb(232, 48, 48), rgb(160, 24, 32)),
        pants: (rgb(248, 200, 32), rgb(200, 144, 16)),
        acc: (rgb(248, 248, 248), rgb(168, 176, 200)),
        power: 3,
        speed: 3,
        catch: 3,
        court: Court::Beach,
        players: [
            PlayerDef { name: "RAD ROD", style: 0, skin: SKIN_LIGHT, hair: BLOND },
            PlayerDef { name: "SURF SUE", style: 5, skin: SKIN_TAN, hair: BLOND },
            PlayerDef { name: "KAHUNA", style: 1, skin: SKIN_DARK, hair: BLACK },
        ],
    },
    TeamDef {
        name: "VENICE VIPERS",
        short: "VIPERS",
        shirt: (rgb(48, 192, 72), rgb(24, 120, 48)),
        pants: (rgb(64, 64, 88), rgb(32, 32, 48)),
        acc: (rgb(248, 232, 40), rgb(184, 152, 16)),
        power: 5,
        speed: 2,
        catch: 2,
        court: Court::Boardwalk,
        players: [
            PlayerDef { name: "SNAKE", style: 2, skin: SKIN_TAN, hair: BLACK },
            PlayerDef { name: "VINNIE", style: 4, skin: SKIN_LIGHT, hair: RED },
            PlayerDef { name: "MAMBA", style: 0, skin: SKIN_DARK, hair: BLACK },
        ],
    },
    TeamDef {
        name: "POWDER PUNKS",
        short: "PUNKS",
        shirt: (rgb(64, 200, 248), rgb(32, 128, 200)),
        pants: (rgb(232, 232, 248), rgb(152, 168, 208)),
        acc: (rgb(248, 72, 168), rgb(176, 32, 112)),
        power: 2,
        speed: 5,
        catch: 2,
        court: Court::Slopes,
        players: [
            PlayerDef { name: "SHRED", style: 3, skin: SKIN_LIGHT, hair: BROWN },
            PlayerDef { name: "YETI", style: 1, skin: SKIN_LIGHT, hair: BLOND },
            PlayerDef { name: "FROSTY", style: 3, skin: SKIN_TAN, hair: BLACK },
        ],
    },
    TeamDef {
        name: "SKATE RATS",
        short: "RATS",
        shirt: (rgb(152, 80, 224), rgb(96, 48, 160)),
        pants: (rgb(248, 136, 32), rgb(192, 88, 16)),
        acc: (rgb(128, 248, 64), rgb(64, 168, 32)),
        power: 2,
        speed: 2,
        catch: 5,
        court: Court::Skatepark,
        players: [
            PlayerDef { name: "OLLIE", style: 4, skin: SKIN_TAN, hair: LIME },
            PlayerDef { name: "GRIND", style: 2, skin: SKIN_DARK, hair: BLACK },
            PlayerDef { name: "KICKFLIP", style: 5, skin: SKIN_LIGHT, hair: PINK },
        ],
    },
];

pub const OUTLINE: u32 = rgb(16, 16, 32);

pub fn palette(team: usize, player: usize) -> Palette {
    let t = &TEAMS[team];
    let p = &t.players[player];
    let mut pal = [0u32; ROLES];
    pal[OUT as usize] = OUTLINE;
    pal[SKIN as usize] = p.skin.0;
    pal[SKIN_D as usize] = p.skin.1;
    pal[HAIR as usize] = p.hair.0;
    pal[HAIR_D as usize] = p.hair.1;
    pal[SHIRT as usize] = t.shirt.0;
    pal[SHIRT_D as usize] = t.shirt.1;
    pal[PANTS as usize] = t.pants.0;
    pal[PANTS_D as usize] = t.pants.1;
    pal[WHITE as usize] = rgb(248, 248, 248);
    pal[ACC as usize] = t.acc.0;
    pal[ACC_D as usize] = t.acc.1;
    pal[SHOE as usize] = rgb(248, 248, 248);
    pal[EYE as usize] = rgb(16, 16, 32);
    pal
}

/// Pale see-through palette for knocked-out players drifting to heaven.
pub fn ghost_palette() -> Palette {
    let mut pal = [rgb(232, 240, 248); ROLES];
    pal[OUT as usize] = rgb(120, 136, 200);
    pal[EYE as usize] = rgb(80, 96, 160);
    pal[SHIRT_D as usize] = rgb(184, 200, 240);
    pal[PANTS_D as usize] = rgb(184, 200, 240);
    pal[SKIN_D as usize] = rgb(200, 216, 240);
    pal
}

/// Flash palette for the hit frame.
pub fn flash_palette() -> Palette {
    let mut pal = [rgb(248, 248, 248); ROLES];
    pal[OUT as usize] = rgb(248, 40, 40);
    pal
}
