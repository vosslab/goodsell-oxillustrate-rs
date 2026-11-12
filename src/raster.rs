//! Integer-offset orthographic sphere samples with source-compatible ownership and clipping.

use crate::{Error, Framing, Scene, SceneSettings};

#[derive(Debug)]
pub(crate) struct FrameBuffer {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth: Vec<f64>,
    pub(crate) atom_index: Vec<Option<usize>>,
    pub(crate) background_depth: f64,
    pub(crate) raw_min: f64,
    pub(crate) raw_max: f64,
}

impl FrameBuffer {
    pub(crate) fn index(&self, row: isize, column: isize) -> Option<usize> {
        if row < 0 || column < 0 || row >= self.height as isize || column >= self.width as isize {
            None
        } else {
            Some(row as usize * self.width as usize + column as usize)
        }
    }

    pub(crate) fn get_depth(&self, row: isize, column: isize) -> f64 {
        self.index(row, column)
            .map_or(self.background_depth, |index| self.depth[index])
    }
}

pub(crate) fn rasterize(scene: &Scene) -> Result<FrameBuffer, Error> {
    let (pixel_count, background_depth) = validate(scene)?;
    let allocation_error = || Error::Allocation {
        width: scene.width,
        height: scene.height,
    };
    pixel_count
        .checked_mul(size_of::<f64>() + size_of::<Option<usize>>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or_else(allocation_error)?;
    let mut depth = Vec::new();
    depth
        .try_reserve_exact(pixel_count)
        .map_err(|_| allocation_error())?;
    depth.resize(pixel_count, background_depth);
    let mut atom_index = Vec::new();
    atom_index
        .try_reserve_exact(pixel_count)
        .map_err(|_| allocation_error())?;
    atom_index.resize(pixel_count, None);

    for (sphere_index, sphere) in scene.spheres.iter().enumerate() {
        let [center_row, center_column, center_z] = sphere.center;
        if center_z >= 0.0 {
            continue;
        }
        let radius_squared = sphere.radius * sphere.radius;
        let (row_start, row_end) = offset_bounds(center_row, sphere.radius, scene.height);
        let (column_start, column_end) = offset_bounds(center_column, sphere.radius, scene.width);
        for row_offset in row_start..=row_end {
            let row_float = center_row + row_offset as f64;
            if row_float < 1.0 || row_float > f64::from(scene.height) {
                continue;
            }
            for column_offset in column_start..=column_end {
                let column_float = center_column + column_offset as f64;
                if column_float < 1.0 || column_float > f64::from(scene.width) {
                    continue;
                }
                let u = row_offset as f64;
                let v = column_offset as f64;
                let distance = (u * u + v * v).sqrt();
                if distance > sphere.radius {
                    continue;
                }
                // Keep the documented sqrt-then-square order for source numerical parity.
                let height = (radius_squared - distance * distance).sqrt();
                let surface_depth = center_z + height;
                if !surface_depth.is_finite() {
                    return Err(Error::Settings(
                        "sphere surface depth must be finite".into(),
                    ));
                }
                let row = row_float as usize - 1;
                let column = column_float as usize - 1;
                let index = row * scene.width as usize + column;
                if surface_depth > depth[index] {
                    depth[index] = surface_depth;
                    atom_index[index] = Some(sphere_index);
                }
            }
        }
    }

    let mut raw_min = f64::INFINITY;
    let mut raw_max = f64::NEG_INFINITY;
    for (value, owner) in depth.iter_mut().zip(&atom_index) {
        if owner.is_some() {
            raw_min = raw_min.min(*value);
            raw_max = raw_max.max(*value);
            *value = value.min(0.0);
        }
    }
    if !raw_min.is_finite() {
        return Err(Error::EmptyScene);
    }
    Ok(FrameBuffer {
        width: scene.width,
        height: scene.height,
        depth,
        atom_index,
        background_depth,
        raw_min,
        raw_max: raw_max.min(0.0),
    })
}

fn offset_bounds(center: f64, radius: f64, dimension: u32) -> (i64, i64) {
    let extent = radius.floor() as i64 + 1;
    // One spare offset preserves samples rounded onto a floating viewport boundary.
    let start = ((1.0 - center).ceil() as i64 - 1).max(-extent);
    let end = ((f64::from(dimension) - center).floor() as i64 + 1).min(extent);
    (start, end)
}

fn validate(scene: &Scene) -> Result<(usize, f64), Error> {
    let settings = SceneSettings {
        width: scene.width,
        height: scene.height,
        dpi: scene.dpi,
        background: scene.background,
        fog: scene.fog.clone(),
        shadows: scene.shadows.clone(),
        outlines: scene.outlines.clone(),
        framing: Framing {
            scale: Some(scene.pixels_per_angstrom),
            ..Framing::default()
        },
        ..SceneSettings::default()
    };
    settings.validate()?;
    let mut background_depth: f64 = -10000.0;
    for sphere in &scene.spheres {
        if sphere.center.iter().any(|value| !value.is_finite())
            || !sphere.radius.is_finite()
            || sphere.radius <= 0.0
            || sphere
                .color
                .iter()
                .any(|value| !(0.0..=1.0).contains(value))
        {
            return Err(Error::Settings(
                "sphere coordinates, positive radius, and unit RGB colors must be finite".into(),
            ));
        }
        // Integer-offset samples require unit spacing to remain representable in f64.
        const INTEGER_LIMIT: f64 = 9_007_199_254_740_992.0;
        if sphere.center[..2]
            .iter()
            .any(|value| value.abs() >= INTEGER_LIMIT)
            || sphere.radius >= INTEGER_LIMIT
        {
            return Err(Error::Settings(
                "sphere pixel coordinates and radius exceed integer sampling precision".into(),
            ));
        }
        background_depth = background_depth.min(sphere.center[2].next_down());
    }
    if !background_depth.is_finite() {
        return Err(Error::Settings(
            "sphere depth leaves no finite background sentinel".into(),
        ));
    }
    Ok((settings.pixel_count()?, background_depth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FogSettings, OutlineSettings, ShadowSettings, Sphere};

    fn sphere(center: [f64; 3], radius: f64) -> Sphere {
        Sphere {
            center,
            radius,
            color: [0.5; 3],
            source_atom_index: 0,
            assembly_instance: 0,
            residue_id: 0,
            chain_id: 0,
            residue_number: None,
        }
    }

    fn scene(spheres: Vec<Sphere>) -> Scene {
        Scene {
            width: 7,
            height: 5,
            dpi: 300,
            spheres,
            background: Some([1.0; 3]),
            fog: FogSettings::default(),
            shadows: ShadowSettings::default(),
            outlines: OutlineSettings::default(),
            pixels_per_angstrom: 1.0,
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn integer_silhouette_has_source_orientation_and_hemisphere_depth() {
        let frame = rasterize(&scene(vec![sphere([3.0, 4.0, -5.0], 2.0)])).unwrap();
        assert_eq!(frame.atom_index.iter().filter(|v| v.is_some()).count(), 13);
        assert_eq!(frame.get_depth(2, 3), -3.0);
        assert_eq!(frame.get_depth(0, 3), -5.0);
        assert!((frame.get_depth(1, 3) - (-5.0 + 3.0_f64.sqrt())).abs() < 1e-14);
        assert_eq!(frame.raw_min, -5.0);
        assert_eq!(frame.raw_max, -3.0);
        assert_eq!(frame.get_depth(-1, 3), frame.background_depth);
        assert_eq!(frame.get_depth(5, 3), frame.background_depth);
        assert_eq!(frame.get_depth(2, 7), frame.background_depth);
        assert_eq!(frame.index(2, 3), Some(17));
    }

    #[test]
    fn fractional_centers_use_integer_offsets_then_float_bounds_then_truncation() {
        let frame = rasterize(&scene(vec![sphere([1.5, 6.5, -5.0], 1.0)])).unwrap();
        let occupied: Vec<_> = frame
            .atom_index
            .iter()
            .enumerate()
            .filter_map(|(index, owner)| owner.map(|_| index))
            .collect();
        assert_eq!(occupied, vec![4, 5, 12]);
        assert_eq!(frame.get_depth(0, 5), -4.0);
        assert_eq!(frame.get_depth(1, 5), -5.0);
    }

    #[test]
    fn raw_depth_selects_nearest_and_first_tie_before_zero_plane_clamp() {
        let frame = rasterize(&scene(vec![
            sphere([3.0, 4.0, -0.5], 1.0),
            sphere([3.0, 4.0, -0.5], 1.0),
            sphere([3.0, 4.0, -0.25], 1.0),
            sphere([3.0, 4.0, 0.0], 100.0),
        ]))
        .unwrap();
        assert_eq!(frame.atom_index[17], Some(2));
        assert_eq!(frame.get_depth(2, 3), 0.0);
        assert_eq!(frame.raw_min, -0.25);
        assert_eq!(frame.raw_max, 0.0);
        let tied = rasterize(&scene(vec![
            sphere([3.0, 4.0, -5.0], 1.0),
            sphere([3.0, 4.0, -5.0], 1.0),
        ]))
        .unwrap();
        assert_eq!(tied.atom_index[17], Some(0));

        let mut cut_face = scene(vec![sphere([1.0, 1.0, -0.5], 2.0)]);
        cut_face.width = 1;
        cut_face.height = 1;
        let cut_face = rasterize(&cut_face).unwrap();
        assert_eq!(cut_face.depth, vec![0.0]);
        assert_eq!(cut_face.raw_min, 1.5);
        assert_eq!(cut_face.raw_max, 0.0);
    }

    #[test]
    fn large_sphere_clips_iteration_and_deep_sphere_gets_farther_background() {
        let frame = rasterize(&scene(vec![sphere([3.0, 4.0, -2e9], 1e9)])).unwrap();
        assert!(frame.atom_index.iter().all(Option::is_some));
        assert!(frame.background_depth < -2e9);
        assert!(
            frame
                .depth
                .iter()
                .all(|depth| *depth > frame.background_depth)
        );
        let deep = rasterize(&scene(vec![sphere([3.0, 4.0, -20000.0], 1.0)])).unwrap();
        assert_eq!(deep.get_depth(2, 3), -19999.0);
        assert!(deep.background_depth < -20000.0);
    }

    #[test]
    fn invisible_and_invalid_public_scenes_return_typed_errors() {
        for spheres in [
            vec![],
            vec![sphere([3.0, 4.0, 0.0], 1.0)],
            vec![sphere([100.0, 100.0, -5.0], 1.0)],
        ] {
            assert!(matches!(rasterize(&scene(spheres)), Err(Error::EmptyScene)));
        }
        let mut invalid = scene(vec![sphere([3.0, 4.0, -5.0], 1.0)]);
        invalid.spheres[0].radius = f64::NAN;
        assert!(matches!(rasterize(&invalid), Err(Error::Settings(_))));
        invalid.spheres[0].radius = 1.0;
        invalid.spheres[0].color[0] = f64::NAN;
        assert!(matches!(rasterize(&invalid), Err(Error::Settings(_))));
        invalid.spheres[0].color = [0.5; 3];
        invalid.spheres[0].center[0] = 1e30;
        assert!(matches!(rasterize(&invalid), Err(Error::Settings(_))));
        invalid.width = u32::MAX;
        assert!(matches!(rasterize(&invalid), Err(Error::Allocation { .. })));
    }
}
