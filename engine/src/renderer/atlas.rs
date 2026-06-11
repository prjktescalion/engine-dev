//! Procedural shape atlas.
//!
//! The Game & Watch aesthetic needs crisp, slightly soft-edged segment shapes
//! (circles, capsules, rounded rects) — not loaded artwork. Instead of
//! shipping image assets, shapes are rasterized at startup on the CPU from
//! signed-distance functions into one grayscale coverage atlas (R8), packed
//! with a simple shelf packer. Sprites then reference a [`Region`] and tint
//! the coverage in the shader, so one texture serves every segment, digit,
//! and indicator.
//!
//! No GPU types here — this module is plain math and `Vec<u8>`, which keeps
//! the rasterizer and the packer unit-testable.

/// A shape to rasterize. Dimensions are in pixels (= design units at 1×).
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Circle {
        radius: f32,
    },
    RoundedRect {
        width: f32,
        height: f32,
        corner: f32,
    },
    /// Capsule of total tip-to-tip `length`, rotated by `angle` radians
    /// counter-clockwise from the +x axis.
    Capsule {
        length: f32,
        radius: f32,
        angle: f32,
    },
}

impl Shape {
    /// Half-extents of the shape's bounding box, before padding.
    fn half_extents(&self) -> (f32, f32) {
        match *self {
            Shape::Circle { radius } => (radius, radius),
            Shape::RoundedRect { width, height, .. } => (width / 2.0, height / 2.0),
            Shape::Capsule {
                length,
                radius,
                angle,
            } => {
                let half = (length / 2.0 - radius).max(0.0);
                (
                    half * angle.cos().abs() + radius,
                    half * angle.sin().abs() + radius,
                )
            }
        }
    }

    /// Signed distance from point `(x, y)` (shape-centered coords) to the
    /// shape surface. Negative inside.
    fn sdf(&self, x: f32, y: f32) -> f32 {
        match *self {
            Shape::Circle { radius } => (x * x + y * y).sqrt() - radius,
            Shape::RoundedRect {
                width,
                height,
                corner,
            } => {
                let qx = x.abs() - (width / 2.0 - corner);
                let qy = y.abs() - (height / 2.0 - corner);
                let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
                outside + qx.max(qy).min(0.0) - corner
            }
            Shape::Capsule {
                length,
                radius,
                angle,
            } => {
                // Rotate the sample into the capsule's local frame, then
                // measure distance to the centerline segment.
                let (s, c) = angle.sin_cos();
                let lx = x * c + y * s;
                let ly = -x * s + y * c;
                let half = (length / 2.0 - radius).max(0.0);
                let dx = lx - lx.clamp(-half, half);
                (dx * dx + ly * ly).sqrt() - radius
            }
        }
    }
}

/// Where a shape landed in the atlas: pixel rect + normalized UVs.
#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub px: (u32, u32),
    pub size: (u32, u32),
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

pub struct Atlas {
    pub width: u32,
    pub height: u32,
    /// R8 coverage, row-major, `width * height` bytes.
    pub pixels: Vec<u8>,
    regions: Vec<Region>,
}

/// Transparent border around each cell so linear sampling never bleeds into
/// a neighbor.
const PAD: u32 = 2;

impl Atlas {
    /// Rasterize and pack `shapes` left-to-right into shelves of `max_width`.
    /// Region order matches input order.
    pub fn build(shapes: &[Shape], max_width: u32) -> Atlas {
        // First pass: cell sizes.
        let cells: Vec<(u32, u32)> = shapes
            .iter()
            .map(|s| {
                let (hx, hy) = s.half_extents();
                ((hx * 2.0).ceil() as u32 + PAD * 2, (hy * 2.0).ceil() as u32 + PAD * 2)
            })
            .collect();

        // Shelf packing.
        let mut placements = Vec::with_capacity(shapes.len());
        let (mut cursor_x, mut cursor_y, mut shelf_h) = (0u32, 0u32, 0u32);
        let mut atlas_w = 0u32;
        for &(w, h) in &cells {
            if cursor_x + w > max_width && cursor_x > 0 {
                cursor_y += shelf_h;
                cursor_x = 0;
                shelf_h = 0;
            }
            placements.push((cursor_x, cursor_y));
            cursor_x += w;
            shelf_h = shelf_h.max(h);
            atlas_w = atlas_w.max(cursor_x);
        }
        let atlas_h = cursor_y + shelf_h;

        let mut pixels = vec![0u8; (atlas_w * atlas_h) as usize];
        let mut regions = Vec::with_capacity(shapes.len());
        for ((shape, &(w, h)), &(x0, y0)) in shapes.iter().zip(&cells).zip(&placements) {
            let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
            for py in 0..h {
                for px in 0..w {
                    // Sample at the pixel center; 1px-wide smoothstep edge.
                    let d = shape.sdf(px as f32 + 0.5 - cx, py as f32 + 0.5 - cy);
                    let coverage = (0.5 - d).clamp(0.0, 1.0);
                    pixels[((y0 + py) * atlas_w + (x0 + px)) as usize] =
                        (coverage * 255.0).round() as u8;
                }
            }
            regions.push(Region {
                px: (x0, y0),
                size: (w, h),
                uv_min: [x0 as f32 / atlas_w as f32, y0 as f32 / atlas_h as f32],
                uv_max: [
                    (x0 + w) as f32 / atlas_w as f32,
                    (y0 + h) as f32 / atlas_h as f32,
                ],
            });
        }

        Atlas {
            width: atlas_w,
            height: atlas_h,
            pixels,
            regions,
        }
    }

    pub fn region(&self, index: usize) -> Region {
        self.regions[index]
    }

    pub fn region_count(&self) -> usize {
        self.regions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(atlas: &Atlas, region: usize, fx: f32, fy: f32) -> u8 {
        let r = atlas.region(region);
        let x = r.px.0 + (fx * r.size.0 as f32) as u32;
        let y = r.px.1 + (fy * r.size.1 as f32) as u32;
        atlas.pixels[(y * atlas.width + x) as usize]
    }

    #[test]
    fn circle_coverage_full_center_empty_corner() {
        let atlas = Atlas::build(&[Shape::Circle { radius: 8.0 }], 256);
        assert_eq!(sample(&atlas, 0, 0.5, 0.5), 255, "center fully covered");
        assert_eq!(sample(&atlas, 0, 0.0, 0.0), 0, "corner outside circle");
    }

    #[test]
    fn rotated_capsule_fits_its_cell() {
        let shape = Shape::Capsule {
            length: 40.0,
            radius: 5.0,
            angle: 0.6,
        };
        let atlas = Atlas::build(&[shape], 256);
        let r = atlas.region(0);
        // Every border pixel of the cell must be empty — the rotated bounds
        // computation must not clip the shape.
        for x in 0..r.size.0 {
            assert_eq!(atlas.pixels[(r.px.1 * atlas.width + r.px.0 + x) as usize], 0);
            let last_row = r.px.1 + r.size.1 - 1;
            assert_eq!(atlas.pixels[(last_row * atlas.width + r.px.0 + x) as usize], 0);
        }
        // And the cell center is inside the capsule body.
        assert_eq!(sample(&atlas, 0, 0.5, 0.5), 255);
    }

    #[test]
    fn packing_never_overlaps_and_uvs_are_normalized() {
        let shapes = vec![
            Shape::Circle { radius: 10.0 },
            Shape::RoundedRect {
                width: 30.0,
                height: 8.0,
                corner: 3.0,
            },
            Shape::Capsule {
                length: 50.0,
                radius: 6.0,
                angle: -1.1,
            },
            Shape::Circle { radius: 3.0 },
        ];
        let atlas = Atlas::build(&shapes, 64); // narrow → forces multiple shelves
        for i in 0..atlas.region_count() {
            let a = atlas.region(i);
            assert!(a.uv_min[0] >= 0.0 && a.uv_max[0] <= 1.0);
            assert!(a.uv_min[1] >= 0.0 && a.uv_max[1] <= 1.0);
            assert!(a.uv_min[0] < a.uv_max[0] && a.uv_min[1] < a.uv_max[1]);
            for j in (i + 1)..atlas.region_count() {
                let b = atlas.region(j);
                let disjoint_x = a.px.0 + a.size.0 <= b.px.0 || b.px.0 + b.size.0 <= a.px.0;
                let disjoint_y = a.px.1 + a.size.1 <= b.px.1 || b.px.1 + b.size.1 <= a.px.1;
                assert!(disjoint_x || disjoint_y, "regions {i} and {j} overlap");
            }
        }
    }

    #[test]
    fn uv_rect_maps_back_to_pixel_rect() {
        let atlas = Atlas::build(&[Shape::Circle { radius: 5.0 }, Shape::Circle { radius: 9.0 }], 256);
        for i in 0..2 {
            let r = atlas.region(i);
            assert_eq!((r.uv_min[0] * atlas.width as f32).round() as u32, r.px.0);
            assert_eq!(
                (r.uv_max[0] * atlas.width as f32).round() as u32,
                r.px.0 + r.size.0
            );
            assert_eq!((r.uv_min[1] * atlas.height as f32).round() as u32, r.px.1);
            assert_eq!(
                (r.uv_max[1] * atlas.height as f32).round() as u32,
                r.px.1 + r.size.1
            );
        }
    }
}
