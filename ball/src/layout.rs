//! LCD panel layout: where every segment lives, and which ones are lit.
//!
//! A real Game & Watch LCD has a fixed set of segments that only ever toggle;
//! motion is an illusion of adjacent segments lighting in sequence. This
//! module builds that fixed set once — ball stations along the arc, the
//! juggler's two arm poses, 7-segment score digits, miss markers — and turns
//! `(scene, game state)` into the frame's sprite list. Every segment is
//! always emitted: unlit ones at [`GHOST_ALPHA`], which is the signature
//! unlit-LCD ghosting.
//!
//! Pure math + tables (shapes go to the engine's atlas; positions are design
//! units), so segment placement and digit encoding are unit-testable.

use engine::renderer::{Atlas, Shape, SpriteInstance};

use crate::game::{Game, Hand, STATIONS};

/// Design space (logical pixels at 1×). The window is opened at an integer
/// multiple of this; the renderer maps design space to the surface.
pub const DESIGN: (f32, f32) = (480.0, 320.0);

/// Ink color of a lit LCD segment (near-black with a green cast).
const INK: [f32; 3] = [0.075, 0.082, 0.07];
/// Alpha of a lit segment.
const LIT_ALPHA: f32 = 0.92;
/// Alpha of an unlit segment — the always-visible ghost outline.
pub const GHOST_ALPHA: f32 = 0.085;

/// Station coordinates: a flat-bottomed arc from the left hand (station 0)
/// over the top to the right hand (station `STATIONS - 1`).
pub fn station_pos(station: usize) -> (f32, f32) {
    let t = station as f32 / (STATIONS - 1) as f32;
    let theta = std::f32::consts::PI * (1.0 - t);
    (240.0 + 160.0 * theta.cos(), 215.0 - 135.0 * theta.sin())
}

/// What turns a segment on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Lit while a ball occupies this station.
    Station(usize),
    /// Lit while the hand pose is Left / Right.
    ArmLeft,
    ArmRight,
    /// Always lit (body, head, ground).
    Always,
    /// One bar of a 7-segment digit. `slot` 0 = tens, 1 = ones.
    Digit { slot: usize, bit: u8 },
    /// Lit when `misses > index`.
    Miss(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    /// Index into the scene's shape list / atlas regions.
    pub shape: usize,
    pub pos: (f32, f32),
    pub role: Role,
}

pub struct Scene {
    pub shapes: Vec<Shape>,
    pub segments: Vec<Segment>,
}

/// Standard 7-segment encodings, bit i = segment i in [a, b, c, d, e, f, g]
/// (a top, b top-right, c bottom-right, d bottom, e bottom-left, f top-left,
/// g middle).
pub const DIGIT_SEGMENTS: [u8; 10] = [
    0x3F, 0x06, 0x5B, 0x4F, 0x66, 0x6D, 0x7D, 0x07, 0x7F, 0x6F,
];

/// Offsets of the 7 bars within a digit cell, matching the bit order above.
/// `(dx, dy, horizontal?)`
const BAR_OFFSETS: [(f32, f32, bool); 7] = [
    (0.0, -16.0, true),  // a
    (9.0, -8.0, false),  // b
    (9.0, 8.0, false),   // c
    (0.0, 16.0, true),   // d
    (-9.0, 8.0, false),  // e
    (-9.0, -8.0, false), // f
    (0.0, 0.0, true),    // g
];

// Shape list indices.
const SH_BALL: usize = 0;
const SH_HBAR: usize = 1;
const SH_VBAR: usize = 2;
const SH_HEAD: usize = 3;
const SH_TORSO: usize = 4;
const SH_ARM_L: usize = 5;
const SH_ARM_R: usize = 6;
const SH_HAND: usize = 7;
const SH_MISS: usize = 8;
const SH_GROUND: usize = 9;

pub fn build_scene() -> Scene {
    let shapes = vec![
        Shape::Circle { radius: 9.0 },                                        // ball
        Shape::RoundedRect { width: 14.0, height: 5.0, corner: 2.0 },         // h bar
        Shape::RoundedRect { width: 5.0, height: 14.0, corner: 2.0 },         // v bar
        Shape::Circle { radius: 12.0 },                                       // head
        Shape::RoundedRect { width: 30.0, height: 44.0, corner: 10.0 },       // torso
        Shape::Capsule { length: 96.0, radius: 6.0, angle: 0.32 },            // arm "\" (to left hand)
        Shape::Capsule { length: 96.0, radius: 6.0, angle: -0.32 },           // arm "/" (to right hand)
        Shape::Circle { radius: 6.5 },                                        // hand cue
        Shape::Circle { radius: 5.0 },                                        // miss dot
        Shape::RoundedRect { width: 360.0, height: 6.0, corner: 3.0 },        // ground line
    ];

    let mut segments = Vec::new();

    // Ball stations along the arc.
    for s in 0..STATIONS {
        segments.push(Segment {
            shape: SH_BALL,
            pos: station_pos(s),
            role: Role::Station(s),
        });
    }

    // The juggler. Head/torso/ground always lit; arms + hand cue per pose.
    segments.push(Segment { shape: SH_HEAD, pos: (240.0, 226.0), role: Role::Always });
    segments.push(Segment { shape: SH_TORSO, pos: (240.0, 262.0), role: Role::Always });
    segments.push(Segment { shape: SH_GROUND, pos: (240.0, 300.0), role: Role::Always });
    segments.push(Segment { shape: SH_ARM_L, pos: (165.0, 240.0), role: Role::ArmLeft });
    segments.push(Segment { shape: SH_HAND, pos: (88.0, 228.0), role: Role::ArmLeft });
    segments.push(Segment { shape: SH_ARM_R, pos: (315.0, 240.0), role: Role::ArmRight });
    segments.push(Segment { shape: SH_HAND, pos: (392.0, 228.0), role: Role::ArmRight });

    // Score: two 7-segment digits, top right (original puts it there).
    for (slot, cx) in [(0usize, 352.0f32), (1, 382.0)] {
        for (bit, &(dx, dy, horizontal)) in BAR_OFFSETS.iter().enumerate() {
            segments.push(Segment {
                shape: if horizontal { SH_HBAR } else { SH_VBAR },
                pos: (cx + dx, 38.0 + dy),
                role: Role::Digit { slot, bit: bit as u8 },
            });
        }
    }

    // Miss markers, top left.
    for m in 0..3 {
        segments.push(Segment {
            shape: SH_MISS,
            pos: (62.0 + 22.0 * m as f32, 38.0),
            role: Role::Miss(m),
        });
    }

    Scene { shapes, segments }
}

/// Build the frame's sprite list: every segment, ghosted or lit.
pub fn sprites(scene: &Scene, atlas: &Atlas, game: &Game) -> Vec<SpriteInstance> {
    let occupied: Vec<usize> = game.ball_stations().collect();
    let tens = (game.score / 10 % 10) as usize;
    let ones = (game.score % 10) as usize;

    scene
        .segments
        .iter()
        .map(|seg| {
            let lit = match seg.role {
                Role::Station(s) => occupied.contains(&s),
                Role::ArmLeft => game.hand == Hand::Left,
                Role::ArmRight => game.hand == Hand::Right,
                Role::Always => true,
                Role::Digit { slot, bit } => {
                    let digit = if slot == 0 { tens } else { ones };
                    DIGIT_SEGMENTS[digit] & (1 << bit) != 0
                }
                Role::Miss(m) => game.misses > m as u32,
            };
            let region = atlas.region(seg.shape);
            SpriteInstance {
                pos: [seg.pos.0, seg.pos.1],
                size: [region.size.0 as f32, region.size.1 as f32],
                uv_min: region.uv_min,
                uv_max: region.uv_max,
                color: [INK[0], INK[1], INK[2], if lit { LIT_ALPHA } else { GHOST_ALPHA }],
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    #[test]
    fn stations_sweep_left_to_right_over_the_top() {
        let xs: Vec<f32> = (0..STATIONS).map(|s| station_pos(s).0).collect();
        assert!(xs.windows(2).all(|w| w[0] < w[1]), "x must increase");
        let (x0, y0) = station_pos(0);
        let (x6, y6) = station_pos(STATIONS - 1);
        assert!((y0 - y6).abs() < 0.001, "hand stations level");
        assert!((x0 - 80.0).abs() < 0.001 && (x6 - 400.0).abs() < 0.001);
        let (_, y_top) = station_pos(STATIONS / 2);
        assert!(y_top < y0, "apex above the hands");
    }

    #[test]
    fn digit_table_is_sane() {
        let mut seen = std::collections::HashSet::new();
        for mask in DIGIT_SEGMENTS {
            assert!(mask < 0x80, "only 7 bits");
            assert!(seen.insert(mask), "digit masks must be distinct");
        }
        assert_eq!(DIGIT_SEGMENTS[8], 0x7F, "8 lights all segments");
        assert_eq!(DIGIT_SEGMENTS[1].count_ones(), 2, "1 is two bars");
    }

    #[test]
    fn scene_has_all_stations_and_both_poses() {
        let scene = build_scene();
        for s in 0..STATIONS {
            assert!(scene.segments.iter().any(|seg| seg.role == Role::Station(s)));
        }
        assert!(scene.segments.iter().any(|s| s.role == Role::ArmLeft));
        assert!(scene.segments.iter().any(|s| s.role == Role::ArmRight));
        // 7 bars per digit slot.
        for slot in 0..2 {
            let bars = scene
                .segments
                .iter()
                .filter(|s| matches!(s.role, Role::Digit { slot: sl, .. } if sl == slot))
                .count();
            assert_eq!(bars, 7);
        }
    }

    #[test]
    fn every_segment_is_drawn_and_ghosting_separates_lit_from_unlit() {
        let scene = build_scene();
        let atlas = Atlas::build(&scene.shapes, 512);
        let game = Game::new();
        let list = sprites(&scene, &atlas, &game);
        assert_eq!(list.len(), scene.segments.len(), "ghosts must be drawn too");

        let lit = list.iter().filter(|s| s.color[3] > 0.5).count();
        let ghosts = list.iter().filter(|s| s.color[3] <= 0.5).count();
        assert_eq!(lit + ghosts, list.len());
        // Fresh game: 2 ball stations, head+torso+ground, one arm pose with
        // its hand cue, digit "00" (6 bars × 2), zero misses.
        assert_eq!(lit, 2 + 3 + 2 + 12);
    }

    #[test]
    fn sprite_uvs_come_from_the_segment_shape_region() {
        let scene = build_scene();
        let atlas = Atlas::build(&scene.shapes, 512);
        let game = Game::new();
        let list = sprites(&scene, &atlas, &game);
        for (seg, sprite) in scene.segments.iter().zip(&list) {
            let r = atlas.region(seg.shape);
            assert_eq!(sprite.uv_min, r.uv_min);
            assert_eq!(sprite.uv_max, r.uv_max);
            assert_eq!(sprite.size, [r.size.0 as f32, r.size.1 as f32]);
        }
    }
}
