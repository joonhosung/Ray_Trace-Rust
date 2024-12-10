use serde::Deserialize;
use crate::elements::sphere::Sphere;
// use crate::elements::Element;
use crate::scene::Member;
use crate::elements::distant_cube_map;
use crate::elements::triangle;
use super::pr;
use keyframe::{Keyframe, AnimationSequence};
use nalgebra::Vector3;
// use keyframe::mint::Vector3;
use keyframe::mint::Point3;
use MemberTypes::*;


#[derive(Deserialize, Debug, Clone)]
pub struct VecInto<T>(Vec<T>); // wrapper st if elements have into one type to another, easily convert this vec into vec of another

impl From<VecInto<MemberTypes>> for Vec<Member<'_>> {
    fn from(mts: VecInto<MemberTypes>) -> Self {
        let mut members: Vec<Member<'_>> = vec![];
        // let mut group_iters: Vec<Box<dyn Iterator<Item = Element>>> = vec![];

        mts.0.into_iter().for_each(|m| {
            use MemberTypes::*;
            // use crate::scene::Member::*;
            match m {
                Sphere(s) => {
                    members.push(Member::Elem(Box::new(s)));
                },
                DistantCubeMap(prcs) => {
                    members.push(Member::Elem(
                        Box::new(distant_cube_map::DistantCubeMap {
                            neg_z: prcs.neg_z.into(),
                            pos_z: prcs.pos_z.into(),
                            neg_x: prcs.neg_x.into(),
                            pos_x: prcs.pos_x.into(),
                            neg_y: prcs.neg_y.into(),
                            pos_y: prcs.pos_y.into(),
                        })));
                },
                FreeTriangle(t) => {
                    members.push(Member::Elem(
                        Box::new(
                            triangle::FreeTriangle {
                                norm: t.norm.normalize().into(),
                                verts: t.verts,
                                rgb: t.rgb,
                                diverts_ray: t.mat,
                            },
                    )));
                },
                Model(m) => {
                    members.extend(m.to_meshes().into_iter().map(|m| Member::Grp(Box::new(m))));
                },
            }
        });

        members
    }
}


// Extract all the locations of the members for each frame
impl VecInto<MemberTypes> {

    pub fn extract_anim(self: VecInto<MemberTypes>, framerate: f32) -> Vec<VecInto<MemberTypes>> {
        
        let max_time: f64 = self.get_last_timestamp() as f64;
        let time_per_frame: f64 = 1.0 / framerate as f64;
        let number_of_frames: usize = (max_time/time_per_frame) as usize;
        let mut frames: Vec<VecInto<MemberTypes>> = Vec::with_capacity(number_of_frames);
        
        for _ in 0..number_of_frames{
            frames.push(VecInto::<MemberTypes>{0: Vec::<MemberTypes>::new()});
        }

        println!("Extracting frames: \n\t Number of frames: {number_of_frames}\n\t Frame vec size: {}\n\t Time per frame {time_per_frame}\n\t Total time: {max_time}s", frames.len());
        self.0.iter().for_each(|m| {            
            match m {
                // Infer the locations of Sphere and Model translations for each frame
                Sphere(s) => {
                    match &s.animation {
                        Some(anim) => {
                            // let hi = anim.keyframes[0].translation.x;
                            

                            let mut sequence = AnimationSequence::<Point3<f32>>::new();
                            for frame in &anim.keyframes {
                                
                                let convert: Point3<f32> = Point3{x: frame.translation.x, y: frame.translation.y, z: frame.translation.z};
                                // s.clone().c
                                sequence.insert(Keyframe::new_dynamic(convert, frame.time, frame.get_ease_type()))
                                    .expect("Something happened while generating keyframe sequence!!");
                            }

                            for i in 0..number_of_frames{
                                let mut frame_to_insert = s.clone();
                                let (x, y, z) = (sequence.now_strict().unwrap().x, sequence.now_strict().unwrap().y, sequence.now_strict().unwrap().z);
                                frame_to_insert.c = Vector3::new(x, y, z);
                                frames[i].0.push(MemberTypes::Sphere(frame_to_insert));
                                sequence.advance_by(time_per_frame);
                            }
                        }, 
                        None => {
                            frames.iter_mut().for_each(|frame| {
                                frame.0.push(MemberTypes::Sphere(s.clone()));
                            }); 
                        },
                    }
                }, 
                Model(m) => {
                    match &m.animation {
                        Some(anim) => {
                            // let hi = anim.keyframes[0].translation.x;
                            

                            let mut sequence = AnimationSequence::<Point3<f32>>::new();
                            for frame in &anim.keyframes {
                                
                                let convert: Point3<f32> = Point3{x: frame.translation.x, y: frame.translation.y, z: frame.translation.z};
                                // s.clone().c
                                sequence.insert(Keyframe::new_dynamic(convert, frame.time, frame.get_ease_type()))
                                    .expect("Something happened while generating keyframe sequence!!");
                            }


                            for i in 0..number_of_frames{
                                let mut frame_to_insert = m.clone();
                                let (x, y, z) = (sequence.now_strict().unwrap().x, sequence.now_strict().unwrap().y, sequence.now_strict().unwrap().z);
                                frame_to_insert.translation = Vector3::new(x, y, z);
                                frames[i].0.push(MemberTypes::Model(frame_to_insert));
                                sequence.advance_by(time_per_frame);
                            } 
                        }, 
                        None => {
                            frames.iter_mut().for_each(|frame| {
                                frame.0.push(MemberTypes::Model(m.clone()));
                            }); 
                        },
                    }
                },

                // No animation for SkyBox or Triangles, but need to copy them into each frame's scene
                FreeTriangle(t) => {
                    frames.iter_mut().for_each(|frame| {
                        frame.0.push(MemberTypes::FreeTriangle(t.clone()));
                    });
                },

                DistantCubeMap(d) => {
                    frames.iter_mut().for_each(|frame| {
                        frame.0.push(MemberTypes::DistantCubeMap(d.clone()));
                    });
                }, 
            }
        });


        

        

        frames
    }

    fn get_last_timestamp(&self) -> f32 {
        let last_timestamp: f32 = self.clone().0.into_iter().map(|m| {
            use MemberTypes::*;
            let mut final_time: f32 = 0.0;
            match m {
                Sphere(s) => {
                    match s.animation {
                        Some(s) => {
                            final_time = s.keyframes.last().unwrap().time;
                            // for frame in s.keyframes {
                                // frame.time 
                            // }
                        }, 
                        None => {},
                    }
                }, 
                Model(m) => {
                    match m.animation {
                        Some(m) => {
                            final_time = m.keyframes.last().unwrap().time;
                        }, 
                        None => {},
                    }
                },

                _ => {}, // No animation for SkyBox or Triangles
            }
            final_time
        }).reduce(f32::max).unwrap();

        last_timestamp
    }
}


impl<A, B> From<VecInto<A>> for Vec<B> 
where
    B: From<A>
{
    fn from(val: VecInto<A>) -> Self {
        let VecInto(contents) = val;
        contents.into_iter().map(|t| t.into()).collect()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub enum MemberTypes {
    Sphere(Sphere),
    DistantCubeMap(pr::DistantCubeMap),
    FreeTriangle(pr::FreeTriangle),

    Model(pr::Model),
}

