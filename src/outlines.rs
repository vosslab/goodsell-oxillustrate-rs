//! Independent depth, subunit, and numeric-residue outline masks.

use crate::raster::FrameBuffer;
use crate::{Scene, Sphere, Thresholds};

pub(crate) fn opacity(scene: &Scene, buffer: &FrameBuffer, row: usize, col: usize) -> f64 {
    let row = row as isize;
    let col = col as isize;
    let settings = &scene.outlines;
    let mut alpha: f64 = 0.0;
    if row >= 1
        && col >= 1
        && row < buffer.height as isize - 1
        && col < buffer.width as isize - 1
        && (settings.subunit_enabled || settings.residue_enabled)
    {
        let center = sphere_at(scene, buffer, row, col);
        let identity = center.map(|sphere| (sphere.chain_id, sphere.assembly_instance));
        let residue = residue_number(center);
        let mut subunit_count = 0;
        let mut residue_count = 0;
        for dr in -2_isize..=2 {
            for dc in -2_isize..=2 {
                if (dr * dc).abs() == 4 {
                    continue;
                }
                let neighbor = sphere_at(scene, buffer, row + dr, col + dc);
                if identity != neighbor.map(|sphere| (sphere.chain_id, sphere.assembly_instance)) {
                    subunit_count += 1;
                }
                if let (Some(p), Some(q)) = (residue, residue_number(neighbor))
                    && (p - q).abs() as f64 > settings.residue_difference
                {
                    residue_count += 1;
                }
            }
        }
        if settings.subunit_enabled {
            alpha = alpha.max(ramp(f64::from(subunit_count), &settings.subunit));
        }
        if settings.residue_enabled {
            alpha = alpha.max(ramp(f64::from(residue_count), &settings.residue));
        }
    }
    if settings.contour_enabled
        && row >= 2
        && col >= 2
        && row < buffer.height as isize - 2
        && col < buffer.width as isize - 2
    {
        alpha = alpha.max(contour(scene, buffer, row, col));
    }
    alpha
}

fn sphere_at<'a>(
    scene: &'a Scene,
    buffer: &FrameBuffer,
    row: isize,
    col: isize,
) -> Option<&'a Sphere> {
    buffer
        .index(row, col)
        .and_then(|index| buffer.atom_index[index])
        .map(|index| &scene.spheres[index])
}

fn residue_number(sphere: Option<&Sphere>) -> Option<i64> {
    match sphere {
        Some(sphere) => sphere.residue_number.map(i64::from),
        None => Some(9999),
    }
}

fn ramp(value: f64, thresholds: &Thresholds) -> f64 {
    ((value - thresholds.low) / (thresholds.high - thresholds.low)).clamp(0.0, 1.0)
}

fn contour(scene: &Scene, buffer: &FrameBuffer, row: isize, col: isize) -> f64 {
    let settings = &scene.outlines;
    if settings.kernel >= 3 {
        let radius: isize = if settings.kernel == 3 { 1 } else { 2 };
        let depth = buffer.get_depth(row, col);
        let minimum = settings.depth_min * scene.pixels_per_angstrom;
        let maximum = settings.depth_max * scene.pixels_per_angstrom;
        let spread = maximum - minimum;
        let mut response = 0.0;
        for dr in -radius..=radius {
            for dc in -radius..=radius {
                if (dr * dc).abs() == 4 {
                    continue;
                }
                let difference = (depth - buffer.get_depth(row + dr, col + dc)).abs();
                if difference > minimum {
                    response += ((difference - minimum) / spread).min(1.0);
                }
            }
        }
        // All nine candidates use the original pixel; the source divides their sum by six.
        return (1.5 * ramp(response, &settings.contour)).min(1.0);
    }
    let mut sum = 0.0;
    let mut positive = 0;
    let mut center = 0.0;
    for dr in -1..=1 {
        for dc in -1..=1 {
            let response = stencil(buffer, row + dr, col + dc, settings.kernel);
            let alpha = ramp(response, &settings.contour);
            sum += alpha;
            positive += usize::from(alpha > 0.0);
            if dr == 0 && dc == 0 {
                center = alpha;
            }
        }
    }
    if positive >= 6 {
        (sum / 6.0).min(1.0)
    } else {
        center
    }
}

fn stencil(buffer: &FrameBuffer, row: isize, col: isize, kernel: u8) -> f64 {
    const SMALL: [[f64; 3]; 3] = [[-0.8, -1.0, -0.8], [-1.0, 7.2, -1.0], [-0.8, -1.0, -0.8]];
    const LARGE: [[f64; 5]; 5] = [
        [0.0, -0.1, -0.2, -0.1, 0.0],
        [-0.1, -0.8, -1.0, -0.8, -0.1],
        [-0.2, -1.0, 8.8, -1.0, -0.2],
        [-0.1, -0.8, -1.0, -0.8, -0.1],
        [0.0, -0.1, -0.2, -0.1, 0.0],
    ];
    let (weights, radius): (&[&[f64]], isize) = if kernel == 1 {
        (&[&SMALL[0], &SMALL[1], &SMALL[2]], 1)
    } else {
        (&[&LARGE[0], &LARGE[1], &LARGE[2], &LARGE[3], &LARGE[4]], 2)
    };
    let mut response = 0.0;
    for (r, weights) in weights.iter().enumerate() {
        for (c, weight) in weights.iter().enumerate() {
            response +=
                weight * buffer.get_depth(row + r as isize - radius, col + c as isize - radius);
        }
    }
    response.abs() / 3.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FogSettings, OutlineSettings, ShadowSettings};

    fn fixture() -> (Scene, FrameBuffer) {
        let sphere = Sphere {
            center: [5.0, 5.0, -5.0],
            radius: 2.0,
            color: [0.5; 3],
            source_atom_index: 0,
            assembly_instance: 0,
            residue_id: 0,
            chain_id: 0,
            residue_number: Some(10),
        };
        let scene = Scene {
            width: 9,
            height: 9,
            spheres: vec![sphere.clone(), sphere],
            background: [1.0; 3],
            fog: FogSettings::default(),
            shadows: ShadowSettings::default(),
            outlines: OutlineSettings {
                contour_enabled: false,
                subunit_enabled: false,
                residue_enabled: false,
                contour: Thresholds {
                    low: 0.1,
                    high: 10.0,
                },
                subunit: Thresholds {
                    low: 0.0,
                    high: 10.0,
                },
                residue: Thresholds {
                    low: 0.0,
                    high: 10.0,
                },
                residue_difference: 5.0,
                ..OutlineSettings::default()
            },
            pixels_per_angstrom: 1.0,
            diagnostics: Vec::new(),
        };
        let buffer = FrameBuffer {
            width: 9,
            height: 9,
            depth: vec![-5.0; 81],
            atom_index: vec![Some(0); 81],
            background_depth: -10000.0,
            raw_min: -5.0,
            raw_max: -5.0,
        };
        (scene, buffer)
    }

    #[test]
    fn chain_and_assembly_changes_count_once_and_dense_residue_ids_do_not_matter() {
        let (mut scene, mut buffer) = fixture();
        scene.outlines.subunit_enabled = true;
        buffer.atom_index[4 * 9 + 5] = Some(1);
        scene.spheres[1].chain_id = 1;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.1);
        scene.spheres[1].assembly_instance = 1;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.1);
        scene.spheres[1].chain_id = 0;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.1);
        scene.spheres[1].assembly_instance = 0;
        scene.spheres[1].residue_id = usize::MAX;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.0);
    }

    #[test]
    fn residues_use_strict_numeric_difference_across_chains_without_overflow() {
        let (mut scene, mut buffer) = fixture();
        scene.outlines.residue_enabled = true;
        buffer.atom_index[4 * 9 + 5] = Some(1);
        scene.spheres[1].chain_id = 1;
        scene.spheres[1].residue_number = Some(15);
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.0);
        scene.spheres[1].residue_number = Some(16);
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.1);
        scene.spheres[1].residue_number = None;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.0);
        scene.spheres[0].residue_number = Some(i32::MIN);
        scene.spheres[1].residue_number = Some(i32::MAX);
        scene.outlines.residue_difference = 4_294_967_294.0;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.1);
        scene.outlines.residue_difference = 4_294_967_295.0;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.0);
    }

    #[test]
    fn identity_border_uses_background_halo_and_background_can_receive_outlines() {
        let (mut scene, mut buffer) = fixture();
        scene.outlines.subunit_enabled = true;
        assert_eq!(opacity(&scene, &buffer, 0, 4), 0.0);
        assert_eq!(opacity(&scene, &buffer, 1, 4), 0.3);
        buffer.atom_index[4 * 9 + 4] = None;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 1.0);
        scene.outlines.subunit_enabled = false;
        scene.outlines.residue_enabled = true;
        scene.outlines.residue_difference = 9989.0;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.0);
        scene.outlines.residue_difference = 9988.0;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 1.0);
        scene.spheres[0].residue_number = None;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.0);
    }

    #[test]
    fn all_kernels_detect_a_step_and_isolated_depth_with_fixed_border() {
        let (mut scene, mut buffer) = fixture();
        scene.outlines.contour_enabled = true;
        for kernel in 1..=4 {
            scene.outlines.kernel = kernel;
            buffer.depth.fill(-5.0);
            assert_eq!(opacity(&scene, &buffer, 4, 4), 0.0);
            for row in 0..9 {
                for col in 5..9 {
                    buffer.depth[row * 9 + col] = 0.0;
                }
            }
            assert!(opacity(&scene, &buffer, 4, 4) > 0.0, "kernel {kernel}");
            assert_eq!(opacity(&scene, &buffer, 1, 4), 0.0);
            buffer.depth.fill(-5.0);
            buffer.depth[4 * 9 + 4] = 0.0;
            assert!(opacity(&scene, &buffer, 4, 4) > 0.0, "kernel {kernel}");
        }
    }

    #[test]
    fn kernel_three_and_four_keep_original_center_boost_and_pixel_depth_scale() {
        let (mut scene, mut buffer) = fixture();
        scene.outlines.contour_enabled = true;
        scene.outlines.contour = Thresholds {
            low: 0.0,
            high: 20.0,
        };
        buffer.depth[4 * 9 + 4] = 0.0;
        scene.outlines.kernel = 3;
        assert!((opacity(&scene, &buffer, 4, 4) - 0.6).abs() < 1e-14);
        scene.outlines.kernel = 4;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 1.0);
        scene.pixels_per_angstrom = 2.0;
        assert!((opacity(&scene, &buffer, 4, 4) - 0.75).abs() < 1e-14);
        scene.outlines.depth_min = 2.5;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.0);
    }

    #[test]
    fn shifted_stencils_use_six_candidate_divisor_or_center_fallback() {
        let (mut scene, mut buffer) = fixture();
        scene.outlines.contour_enabled = true;
        scene.outlines.contour = Thresholds {
            low: 0.0,
            high: 100.0,
        };
        buffer.depth[4 * 9 + 4] = 0.0;
        scene.outlines.kernel = 1;
        assert!((opacity(&scene, &buffer, 4, 4) - 0.04).abs() < 1e-14);
        scene.outlines.kernel = 2;
        assert!((opacity(&scene, &buffer, 4, 4) - 2.0 / 45.0).abs() < 1e-14);
        scene.outlines.kernel = 1;
        scene.outlines.contour.low = 1.5;
        // Only five candidates exceed this threshold, so use the center response of 12.
        assert!((opacity(&scene, &buffer, 4, 4) - 10.5 / 98.5).abs() < 1e-14);
    }

    #[test]
    fn shifted_large_stencil_reads_deterministic_halo_and_passes_combine_by_maximum() {
        let (mut scene, mut buffer) = fixture();
        scene.outlines.contour_enabled = true;
        scene.outlines.kernel = 2;
        let edge = opacity(&scene, &buffer, 2, 4);
        assert!(stencil(&buffer, 1, 4, 2) > 0.0);
        assert_eq!(edge, 0.0);
        assert_eq!(edge, opacity(&scene, &buffer, 2, 4));
        buffer.depth[4 * 9 + 4] = 0.0;
        buffer.atom_index[4 * 9 + 5] = Some(1);
        scene.spheres[1].chain_id = 1;
        scene.spheres[1].residue_number = Some(100);
        let contour_alpha = opacity(&scene, &buffer, 4, 4);
        scene.outlines.contour_enabled = false;
        scene.outlines.subunit_enabled = true;
        let subunit_alpha = opacity(&scene, &buffer, 4, 4);
        scene.outlines.subunit_enabled = false;
        scene.outlines.residue_enabled = true;
        let residue_alpha = opacity(&scene, &buffer, 4, 4);
        scene.outlines.contour_enabled = true;
        scene.outlines.subunit_enabled = true;
        assert_eq!(
            opacity(&scene, &buffer, 4, 4),
            contour_alpha.max(subunit_alpha).max(residue_alpha)
        );
        scene.outlines.contour_enabled = false;
        scene.outlines.subunit_enabled = false;
        scene.outlines.residue_enabled = false;
        assert_eq!(opacity(&scene, &buffer, 4, 4), 0.0);
    }
}
