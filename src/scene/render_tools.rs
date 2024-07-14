use nalgebra::{Vector3, vector};
use std::iter::zip;
use crate::render_target::RenderTarget;
use crate::ray::{RayCompute, Hitable, HasHitInfo};
use super::Scene;

pub fn render_to_target(render_target: &RenderTarget, scene: &Scene) {
    use rayon::prelude::*;

    let ray_compute = RayCompute::new(&render_target, &scene.cam);

    use std::time::Instant;
    let start = Instant::now();
    render_target.buff_mux.lock()
        .par_chunks_mut(4) // pixels have rgba values, so chunk by 4
        .enumerate()
        .map(|(i, pix)| (render_target.chunk_to_pix(i.try_into().unwrap()), pix))
        .for_each(|((x, y), pix)| {
            let ray = ray_compute.pix_cam_to_ray((x,y), &scene.cam);

            let hit_results: Vec<_> = scene.objs.iter().map(|sph| sph.intersect(&ray)).collect();
            
            let obj_w_hit = zip(&scene.objs, &hit_results)
                .filter_map(|(o, hro)| {
                    match hro {
                        Some(hr) => Some((o, hr)),
                        None => None,
                    }
                })
                .min_by_key(|(_, hr)| hr.l.clone()); // closest hit result found here
            
            let rgb = if let Some((obj, hit_result)) = obj_w_hit { obj.hit_info(hit_result).rgb } else { vector![0.0, 0.0, 0.0] };

            // use std::collections::HashSet;
            // let check_set = HashSet::from([(0,0), (1000,0)]);
            // if let Some((_obj, hit_result)) = obj_w_hit {
            //     if check_set.contains(&(x,y)) {
            //         let fuck: f32 = hit_result.l.clone().into();
            //         println!("pixel: {:?}, ray len: {}", (x,y), fuck);
            //     }
            // }

            // let dat: [u8; 4] = [200, 0, 100, 0];
            pix.copy_from_slice(&rgb_f_to_u8(&rgb));
        });
    let elapsed = start.elapsed();
    println!("elapsed {:?}", elapsed);
}

fn rgb_f_to_u8(f: &Vector3<f32>) -> [u8; 4] {
    let mut out: [u8; 4] = [0; 4];
    zip(out.iter_mut(), f.iter()).for_each(|(e, f)| *e = (f * 255.0).trunc() as u8); // assume 0.0 -> 1.0 range
    out
}