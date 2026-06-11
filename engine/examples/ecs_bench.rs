//! Benchmark: old wireframe `Vec<Entity>` world vs the sparse-set core.
//!
//! Run with: cargo run --release -p engine --example ecs_bench
//!
//! Three workloads, all taken from what the studio actually does:
//! - spawn:     build a world of N entities, half of them moving
//! - tick:      600 frames of the movement system (10s at 60 fps)
//! - writeback: 60 frames of per-entity transform lookup by id (the studio's
//!   scene sync). This is where the old core's O(n) `find` goes quadratic.

use std::hint::black_box;
use std::time::Instant;

use engine::ecs::{EntityId, Transform, Velocity, World};

// ----- Old core, reproduced verbatim as the baseline ------------------------

mod naive {
    use engine::ecs::{Transform, Velocity};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EntityId(pub u64);

    pub struct Entity {
        pub id: EntityId,
        // Never read, but kept so the baseline carries the same per-entity
        // weight as the original wireframe world.
        #[allow(dead_code)]
        pub name: String,
        pub transform: Transform,
        pub velocity: Option<Velocity>,
    }

    pub struct NaiveWorld {
        pub entities: Vec<Entity>,
        next_id: u64,
        pub bounds: (f32, f32, f32, f32),
    }

    impl NaiveWorld {
        pub fn new() -> Self {
            Self {
                entities: Vec::new(),
                next_id: 1,
                bounds: (0.0, 0.0, 1024.0, 600.0),
            }
        }

        pub fn spawn(&mut self, name: String) -> EntityId {
            let id = EntityId(self.next_id);
            self.next_id += 1;
            self.entities.push(Entity {
                id,
                name,
                transform: Transform::default(),
                velocity: None,
            });
            id
        }

        pub fn get(&self, id: EntityId) -> Option<&Entity> {
            self.entities.iter().find(|e| e.id == id)
        }

        pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
            self.entities.iter_mut().find(|e| e.id == id)
        }

        pub fn run_systems(&mut self, dt: f32) {
            let (min_x, min_y, max_x, max_y) = self.bounds;
            for e in &mut self.entities {
                let Some(v) = e.velocity.as_mut() else { continue };
                e.transform.x += v.vx * dt;
                e.transform.y += v.vy * dt;
                e.transform.rotation += v.vrot * dt;
                if e.transform.x < min_x {
                    e.transform.x = min_x;
                    v.vx = v.vx.abs();
                } else if e.transform.x > max_x {
                    e.transform.x = max_x;
                    v.vx = -v.vx.abs();
                }
                if e.transform.y < min_y {
                    e.transform.y = min_y;
                    v.vy = v.vy.abs();
                } else if e.transform.y > max_y {
                    e.transform.y = max_y;
                    v.vy = -v.vy.abs();
                }
            }
        }
    }
}

// ----- Workloads -------------------------------------------------------------

const TICK_FRAMES: u32 = 600;
const WRITEBACK_FRAMES: u32 = 60;
/// Above this, the naive writeback (O(n²) per frame) takes too long to wait on.
const NAIVE_WRITEBACK_CAP: usize = 20_000;

fn velocity_for(i: usize) -> Velocity {
    Velocity {
        vx: 30.0 + (i % 100) as f32,
        vy: 20.0 + (i % 77) as f32,
        vrot: 0.5,
    }
}

fn place(t: &mut Transform, i: usize) {
    t.x = (i % 1000) as f32;
    t.y = (i % 600) as f32;
}

fn build_naive(n: usize) -> (naive::NaiveWorld, Vec<naive::EntityId>) {
    let mut w = naive::NaiveWorld::new();
    let ids: Vec<_> = (0..n).map(|i| w.spawn(format!("e{i}"))).collect();
    for (i, &id) in ids.iter().enumerate() {
        let e = w.get_mut(id).unwrap();
        place(&mut e.transform, i);
        if i % 2 == 0 {
            e.velocity = Some(velocity_for(i));
        }
    }
    (w, ids)
}

fn build_sparse(n: usize) -> (World, Vec<EntityId>) {
    let mut w = World::new();
    let ids: Vec<_> = (0..n).map(|i| w.spawn(format!("e{i}"))).collect();
    for (i, &id) in ids.iter().enumerate() {
        place(w.transform_mut(id).unwrap(), i);
        if i % 2 == 0 {
            w.set_velocity(id, velocity_for(i));
        }
    }
    (w, ids)
}

fn time<R>(f: impl FnOnce() -> R) -> (R, f64) {
    let start = Instant::now();
    let r = f();
    (r, start.elapsed().as_secs_f64() * 1000.0)
}

fn row(label: &str, naive_ms: Option<f64>, sparse_ms: f64) {
    match naive_ms {
        Some(n) => println!(
            "  {label:<28} {n:>10.3} ms {sparse_ms:>10.3} ms {:>8.1}x",
            n / sparse_ms
        ),
        None => println!("  {label:<28} {:>10} {sparse_ms:>10.3} ms {:>8}", "skipped", "—"),
    }
}

fn main() {
    println!("ECS bench — naive Vec<Entity> vs sparse-set core");
    println!(
        "workloads: spawn N | {TICK_FRAMES} ticks | {WRITEBACK_FRAMES}-frame writeback (per-id lookup)\n"
    );

    for n in [1_000usize, 10_000, 100_000] {
        println!("N = {n} entities ({} moving)", n / 2);
        println!("  {:<28} {:>13} {:>13} {:>9}", "", "naive", "sparse-set", "speedup");

        let (_, naive_spawn) = time(|| black_box(build_naive(n)));
        let (_, sparse_spawn) = time(|| black_box(build_sparse(n)));
        row("spawn + init", Some(naive_spawn), sparse_spawn);

        let (mut nw, _) = build_naive(n);
        let (_, naive_tick) = time(|| {
            for _ in 0..TICK_FRAMES {
                nw.run_systems(1.0 / 60.0);
            }
            black_box(&nw);
        });
        let (mut sw, _) = build_sparse(n);
        let (_, sparse_tick) = time(|| {
            for _ in 0..TICK_FRAMES {
                sw.run_systems(1.0 / 60.0);
            }
            black_box(&sw);
        });
        row(
            &format!("tick x{TICK_FRAMES}"),
            Some(naive_tick),
            sparse_tick,
        );

        let naive_wb = (n <= NAIVE_WRITEBACK_CAP).then(|| {
            let (nw, ids) = build_naive(n);
            let (sum, ms) = time(|| {
                let mut acc = 0.0f32;
                for _ in 0..WRITEBACK_FRAMES {
                    for &id in &ids {
                        let e = nw.get(id).unwrap();
                        acc += e.transform.x + e.transform.y;
                    }
                }
                acc
            });
            black_box(sum);
            ms
        });
        let (sw, ids) = build_sparse(n);
        let (sum, sparse_wb) = time(|| {
            let mut acc = 0.0f32;
            for _ in 0..WRITEBACK_FRAMES {
                for &id in &ids {
                    let t = sw.transform(id).unwrap();
                    acc += t.x + t.y;
                }
            }
            acc
        });
        black_box(sum);
        row(
            &format!("writeback x{WRITEBACK_FRAMES}"),
            naive_wb,
            sparse_wb,
        );
        println!();
    }
}
