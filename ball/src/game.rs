//! Game & Watch *Ball* (1980) rules on the engine ECS.
//!
//! Two balls bounce along a fixed arc of [`STATIONS`] discrete stations
//! (0 = left hand, `STATIONS - 1` = right hand), one station per tick, out of
//! phase. The player's hand pose (Left/Right) decides each arrival: right
//! pose at the right station is a catch (score++, ball tossed back), wrong
//! pose is a miss. Three misses ends the game; the tick clock speeds up with
//! score.
//!
//! ECS usage: balls are entities in the engine's sparse-set [`World`] (their
//! `Transform` mirrors the station each tick); the game-specific component —
//! [`BallState`] — lives in a game-owned [`SparseSet`] keyed by the same
//! `EntityId`s, exactly how the world's fixed roster was designed to be
//! extended from outside. No render types in this module, so every rule is
//! unit-tested headless.

use std::time::Duration;

use engine::ecs::{EntityId, SparseSet, World};

/// Number of stations on the arc, hands included.
pub const STATIONS: usize = 7;
pub const MAX_MISSES: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

/// Per-ball component: where on the arc, and which way it's travelling.
/// Invariant: `dir` flips on arrival at either end, so `station` stays in
/// `0..STATIONS`.
#[derive(Debug, Clone, Copy)]
pub struct BallState {
    pub station: usize,
    dir: i32,
}

/// (station, dir) for each ball at game start. Ball 1 leads ball 0 by half a
/// bounce, so arrivals come every 3 ticks in an R R L L rhythm.
const INITIAL: [(usize, i32); 2] = [(0, 1), (3, 1)];

pub struct Game {
    pub world: World,
    /// Game-side component store, keyed by world entity ids.
    pub ball_states: SparseSet<BallState>,
    pub hand: Hand,
    pub score: u32,
    pub misses: u32,
    pub ticks: u64,
    pub over: bool,
}

impl Game {
    pub fn new() -> Self {
        let mut world = World::new();
        let mut ball_states = SparseSet::new();
        for (i, &(station, dir)) in INITIAL.iter().enumerate() {
            let id = world.spawn(format!("ball-{i}"));
            ball_states.insert(id, BallState { station, dir });
        }
        Self {
            world,
            ball_states,
            hand: Hand::Left,
            score: 0,
            misses: 0,
            ticks: 0,
            over: false,
        }
    }

    pub fn set_hand(&mut self, hand: Hand) {
        self.hand = hand;
    }

    /// Advance every ball one station and resolve arrivals.
    pub fn tick(&mut self) {
        if self.over {
            return;
        }
        self.ticks += 1;
        let (mut caught, mut missed) = (0u32, 0u32);
        for (_, ball) in self.ball_states.iter_mut() {
            ball.station = (ball.station as i32 + ball.dir) as usize;
            let catch_hand = if ball.station == 0 {
                Some(Hand::Left)
            } else if ball.station == STATIONS - 1 {
                Some(Hand::Right)
            } else {
                None
            };
            if let Some(required) = catch_hand {
                if required == self.hand {
                    caught += 1;
                } else {
                    missed += 1;
                }
                // Caught balls are tossed back; missed ones bounce anyway so
                // the round keeps flowing (the penalty is the miss mark).
                ball.dir = -ball.dir;
            }
        }
        self.score += caught;
        self.misses += missed;
        if self.misses >= MAX_MISSES {
            self.over = true;
        }
    }

    /// Time between ticks: starts leisurely, tightens 6 ms per point down to
    /// a 140 ms floor.
    pub fn tick_interval(&self) -> Duration {
        Duration::from_millis(420u64.saturating_sub(6 * self.score as u64).max(140))
    }

    /// Fresh round; the hand pose is kept (you're still holding the console).
    pub fn restart(&mut self) {
        let hand = self.hand;
        *self = Self::new();
        self.hand = hand;
    }

    /// Stations currently occupied by balls (dense iteration over the store).
    pub fn ball_stations(&self) -> impl Iterator<Item = usize> + '_ {
        self.ball_states.iter().map(|(_, b)| b.station)
    }

    /// Mirror each ball's station into its world `Transform` so the ECS view
    /// of the scene matches the LCD (and anything else reading the world —
    /// future debug overlays, tests — sees real positions).
    pub fn sync_transforms(&mut self, station_pos: impl Fn(usize) -> (f32, f32)) {
        let stations: Vec<(EntityId, usize)> = self
            .ball_states
            .iter()
            .map(|(id, b)| (id, b.station))
            .collect();
        for (id, station) in stations {
            if let Some(t) = self.world.transform_mut(id) {
                let (x, y) = station_pos(station);
                t.x = x;
                t.y = y;
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_has_two_balls_in_world_and_store() {
        let g = Game::new();
        assert_eq!(g.world.len(), 2);
        assert_eq!(g.ball_states.len(), 2);
        assert_eq!(g.score, 0);
        assert_eq!(g.misses, 0);
        assert!(!g.over);
    }

    #[test]
    fn balls_advance_one_station_per_tick_and_stay_in_range() {
        let mut g = Game::new();
        let before: Vec<_> = g.ball_stations().collect();
        g.tick();
        let after: Vec<_> = g.ball_stations().collect();
        for (b, a) in before.iter().zip(&after) {
            assert_eq!((*a as i32 - *b as i32).abs(), 1);
        }
        for _ in 0..100 {
            g.set_hand(if g.ticks % 6 < 3 { Hand::Right } else { Hand::Left });
            g.tick();
            assert!(g.ball_stations().all(|s| s < STATIONS), "station escaped the arc");
        }
    }

    #[test]
    fn rrll_rhythm_scores_with_correct_hands() {
        let mut g = Game::new();
        // Arrivals: t3 right, t6 right, t9 left, t12 left.
        g.set_hand(Hand::Right);
        for _ in 0..6 {
            g.tick();
        }
        assert_eq!((g.score, g.misses), (2, 0));
        g.set_hand(Hand::Left);
        for _ in 0..6 {
            g.tick();
        }
        assert_eq!((g.score, g.misses), (4, 0));
        assert!(!g.over);
    }

    #[test]
    fn wrong_hand_misses_and_three_misses_end_the_game() {
        let mut g = Game::new();
        // Hand stays Left: t3 and t6 arrive right (miss, miss), t9 and t12
        // arrive left (catch), t15 arrives right (third miss) — game over.
        for _ in 0..15 {
            g.tick();
        }
        assert_eq!(g.misses, MAX_MISSES);
        assert_eq!(g.score, 2);
        assert!(g.over);
        // Frozen after game over.
        let (ticks, stations): (u64, Vec<_>) = (g.ticks, g.ball_stations().collect());
        g.tick();
        assert_eq!(g.ticks, ticks);
        assert_eq!(g.ball_stations().collect::<Vec<_>>(), stations);
    }

    #[test]
    fn tick_interval_ramps_down_to_a_floor() {
        let mut g = Game::new();
        let start = g.tick_interval();
        g.score = 20;
        let mid = g.tick_interval();
        g.score = 10_000;
        let floor = g.tick_interval();
        assert!(start > mid && mid > floor);
        assert_eq!(floor, Duration::from_millis(140));
    }

    #[test]
    fn restart_resets_state_but_keeps_hand() {
        let mut g = Game::new();
        g.set_hand(Hand::Right);
        for _ in 0..20 {
            g.tick();
        }
        g.restart();
        assert_eq!((g.score, g.misses, g.ticks), (0, 0, 0));
        assert!(!g.over);
        assert_eq!(g.hand, Hand::Right);
        assert_eq!(g.world.len(), 2);
    }

    #[test]
    fn sync_transforms_mirrors_stations_into_the_world() {
        let mut g = Game::new();
        g.tick();
        g.sync_transforms(|s| (s as f32 * 10.0, 99.0));
        for (id, b) in g.ball_states.iter() {
            let t = g.world.transform(id).unwrap();
            assert_eq!(t.x, b.station as f32 * 10.0);
            assert_eq!(t.y, 99.0);
        }
    }
}
