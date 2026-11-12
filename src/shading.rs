//! Deterministic screen-space conical shadows and molecular depth cueing.

use crate::Scene;
use crate::raster::FrameBuffer;

pub(crate) struct Shading<'a> {
    scene: &'a Scene,
    buffer: &'a FrameBuffer,
    samples: [(isize, isize, f64); 441],
}

impl<'a> Shading<'a> {
    pub(crate) fn new(scene: &'a Scene, buffer: &'a FrameBuffer) -> Self {
        let samples = std::array::from_fn(|index| {
            let row = (index / 21) as isize * 5 - 50;
            let col = (index % 21) as isize * 5 - 50;
            let radius = ((row * row + col * col) as f64).sqrt();
            let distance = if radius > 50.0 || (row == 0 && col == 0) {
                10000.0
            } else {
                radius
            };
            (row, col, distance)
        });
        Self {
            scene,
            buffer,
            samples,
        }
    }

    pub(crate) fn color(&self, row: usize, col: usize) -> [f64; 3] {
        let index = row * self.buffer.width as usize + col;
        let Some(atom) = self.buffer.atom_index[index] else {
            return self.scene.background.unwrap_or([0.0; 3]);
        };
        let shadow = self.shadow_factor(row, col);
        let weight = self.fog_weight(self.buffer.depth[index]);
        std::array::from_fn(|channel| {
            weight * shadow * self.scene.spheres[atom].color[channel]
                + (1.0 - weight) * self.scene.fog.color[channel]
        })
    }

    fn shadow_factor(&self, row: usize, col: usize) -> f64 {
        let index = row * self.buffer.width as usize + col;
        if !self.scene.shadows.enabled || self.buffer.atom_index[index].is_none() {
            return 1.0;
        }
        let settings = &self.scene.shadows;
        let mut shadow: f64 = 1.0;
        for &(row_offset, col_offset, distance) in &self.samples {
            let sample_row = row as isize + row_offset;
            let sample_col = col as isize + col_offset;
            // The source's strict 1-based upper bound excludes the last row and column.
            if sample_row < 0
                || sample_col < 0
                || sample_row >= self.buffer.height as isize - 1
                || sample_col >= self.buffer.width as isize - 1
            {
                continue;
            }
            let dz = self.buffer.get_depth(sample_row, sample_col) - self.buffer.depth[index];
            if dz > settings.threshold && distance * settings.slope < dz + settings.threshold {
                shadow -= settings.decrement;
            }
        }
        shadow.max(settings.brightness_floor)
    }

    fn fog_weight(&self, depth: f64) -> f64 {
        let fog = &self.scene.fog;
        if depth < self.buffer.raw_min {
            return 1.0;
        }
        let spread = self.buffer.raw_max - self.buffer.raw_min;
        if spread == 0.0 {
            return fog.front;
        }
        fog.front - (self.buffer.raw_max - depth) / spread * (fog.front - fog.back)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FogSettings, OutlineSettings, ShadowSettings, Sphere};

    fn scene() -> Scene {
        Scene {
            width: 111,
            height: 111,
            spheres: vec![Sphere {
                center: [0.0; 3],
                radius: 1.0,
                color: [0.8, 0.4, 0.2],
                source_atom_index: 0,
                assembly_instance: 0,
                residue_id: 0,
                chain_id: 0,
                residue_number: None,
            }],
            background: Some([0.1, 0.2, 0.3]),
            fog: FogSettings {
                color: [1.0; 3],
                front: 0.8,
                back: 0.2,
            },
            shadows: ShadowSettings {
                decrement: 0.1,
                slope: 2.0,
                threshold: 1.0,
                brightness_floor: 0.0,
                ..ShadowSettings::default()
            },
            outlines: OutlineSettings::default(),
            pixels_per_angstrom: 12.0,
            diagnostics: Vec::new(),
        }
    }

    fn buffer() -> FrameBuffer {
        let mut buffer = FrameBuffer {
            width: 111,
            height: 111,
            depth: vec![-1000.0; 111 * 111],
            atom_index: vec![None; 111 * 111],
            background_depth: -1000.0,
            raw_min: -200.0,
            raw_max: 0.0,
        };
        occupy(&mut buffer, 55, 55, -200.0);
        buffer
    }

    fn occupy(buffer: &mut FrameBuffer, row: usize, col: usize, depth: f64) {
        let index = row * buffer.width as usize + col;
        buffer.depth[index] = depth;
        buffer.atom_index[index] = Some(0);
    }

    fn close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
    }

    #[test]
    fn fixed_lattice_preserves_circle_and_self_sentinels() {
        let scene = scene();
        let buffer = buffer();
        let shading = Shading::new(&scene, &buffer);
        assert_eq!(shading.samples.len(), 441);
        assert_eq!(shading.samples[0], (-50, -50, 10000.0));
        assert_eq!(shading.samples[220], (0, 0, 10000.0));
        assert!(shading.samples.contains(&(0, 50, 50.0)));
        assert!(shading.samples.contains(&(30, 40, 50.0)));
        assert!(shading.samples.contains(&(35, 40, 10000.0)));
    }

    #[test]
    fn both_cone_inequalities_are_strict_and_threshold_is_pixels() {
        let scene = scene();
        let mut buffer = buffer();
        // At distance 5, cone equality is dz + threshold == 10.
        occupy(&mut buffer, 55, 60, -191.0);
        close(Shading::new(&scene, &buffer).shadow_factor(55, 55), 1.0);
        occupy(&mut buffer, 55, 60, -190.999);
        close(Shading::new(&scene, &buffer).shadow_factor(55, 55), 0.9);
        let mut scene = scene;
        scene.shadows.slope = 0.0;
        occupy(&mut buffer, 55, 60, -199.0);
        close(Shading::new(&scene, &buffer).shadow_factor(55, 55), 1.0);
        occupy(&mut buffer, 55, 60, -198.999);
        close(Shading::new(&scene, &buffer).shadow_factor(55, 55), 0.9);
    }

    #[test]
    fn radius_sentinel_is_a_distance_not_an_omitted_sample() {
        let mut scene = scene();
        let mut buffer = buffer();
        occupy(&mut buffer, 5, 5, 0.0);
        close(Shading::new(&scene, &buffer).shadow_factor(55, 55), 1.0);
        scene.shadows.slope = 0.0;
        close(Shading::new(&scene, &buffer).shadow_factor(55, 55), 0.9);
    }

    #[test]
    fn sampling_includes_first_row_and_column_but_excludes_last() {
        let mut scene = scene();
        scene.shadows.slope = 0.0;
        let mut buffer = buffer();
        occupy(&mut buffer, 50, 50, -200.0);
        occupy(&mut buffer, 0, 50, 0.0);
        occupy(&mut buffer, 50, 0, 0.0);
        close(Shading::new(&scene, &buffer).shadow_factor(50, 50), 0.8);
        occupy(&mut buffer, 60, 60, -200.0);
        occupy(&mut buffer, 110, 60, 0.0);
        occupy(&mut buffer, 60, 110, 0.0);
        close(Shading::new(&scene, &buffer).shadow_factor(60, 60), 1.0);
    }

    #[test]
    fn brightness_floor_disabled_shadows_and_background() {
        let mut scene = scene();
        scene.shadows.brightness_floor = 0.85;
        let mut buffer = buffer();
        occupy(&mut buffer, 55, 60, 0.0);
        occupy(&mut buffer, 60, 55, 0.0);
        close(Shading::new(&scene, &buffer).shadow_factor(55, 55), 0.85);
        close(Shading::new(&scene, &buffer).shadow_factor(55, 50), 1.0);
        assert_eq!(
            Shading::new(&scene, &buffer).color(55, 50),
            scene.background.unwrap()
        );
        scene.shadows.enabled = false;
        close(Shading::new(&scene, &buffer).shadow_factor(55, 55), 1.0);
    }

    #[test]
    fn fog_uses_raw_extrema_flat_guard_and_unclamped_weights() {
        let scene = scene();
        let mut buffer = buffer();
        let shading = Shading::new(&scene, &buffer);
        close(shading.fog_weight(-200.0), 0.2);
        close(shading.fog_weight(-100.0), 0.5);
        close(shading.fog_weight(0.0), 0.8);
        close(shading.fog_weight(-1000.0), 1.0);
        buffer.raw_min = 5.0;
        buffer.raw_max = 0.0;
        close(Shading::new(&scene, &buffer).fog_weight(0.0), 1.0);
        buffer.raw_min = -100.0;
        buffer.raw_max = -100.0;
        close(Shading::new(&scene, &buffer).fog_weight(-100.0), 0.8);
        buffer.raw_min = -200.0;
        close(Shading::new(&scene, &buffer).fog_weight(0.0), 1.4);
        let mut scene = scene;
        scene.fog.front = 0.2;
        scene.fog.back = 0.8;
        close(Shading::new(&scene, &buffer).fog_weight(-150.0), 0.5);
    }

    #[test]
    fn color_shadows_molecule_before_adding_fog() {
        let scene = scene();
        let mut buffer = buffer();
        occupy(&mut buffer, 55, 55, -100.0);
        occupy(&mut buffer, 55, 60, 0.0);
        let shading = Shading::new(&scene, &buffer);
        for (actual, base) in shading.color(55, 55).into_iter().zip([0.8, 0.4, 0.2]) {
            close(actual, 0.5 * 0.9 * base + 0.5);
        }
        assert_eq!(shading.color(0, 0), scene.background.unwrap());
    }
}
