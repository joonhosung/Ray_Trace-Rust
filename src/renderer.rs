use std::thread;
use std::sync::Arc;
use egui::mutex::Mutex;
use crate::builder::inner::{MemberTypes, VecInto};
use crate::scene::{Scene, GPUScene};
use std::sync::mpsc::{channel, Sender, Receiver};
pub use crate::render::{RenderTarget, render_to_target_cpu, render_to_target_gpu};
pub use crate::builder::Scheme;
use crate::ui_util;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;
use std::time::Instant;

pub struct RenderOut {
    pub buf_q: Receiver<Vec<u8>>,
}

#[derive(Clone)]
pub struct Renderer {
    target: RenderTarget,
    sender: Sender<Vec<u8>>,

    scheme: Scheme,
}

impl Renderer {
    pub fn new(canv_width: i32, canv_height: i32, scheme: Scheme) -> (Self, Receiver<Vec<u8>>) {
        let buf: Vec<u8> = [0, 0, 0, 0].repeat((canv_width * canv_height).try_into().unwrap());
        let (tx, rx) = channel();
        let target = RenderTarget {
            buff_mux: Arc::new(Mutex::new(buf)),
            canv_width, canv_height,
        };
        (Self {
            target,
            sender: tx,
            scheme: scheme,
        }, rx)
    }

    pub fn consume_and_do(self) {
        // let start = Instant::now();
        let use_gpu = self.scheme.render_info.use_gpu.unwrap_or(false);
        thread::spawn(move || {
            let renderer_inner = self.clone();
            let iter_progress = ProgressBar::new(self.scheme.render_info.samps_per_pix as u64);
            iter_progress.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:80.cyan/80.blue} {pos}/{len} {msg}").unwrap()
            );
            if !use_gpu {
                let skene = Scene { cam: renderer_inner.scheme.cam.into(), members: renderer_inner.scheme.scene_members.into() };
                render_to_target_cpu(&self.target, &skene, || self.update_output(), &self.scheme.render_info, &iter_progress);
            }
            else {
                let batch_size = self.scheme.render_info.gpu_render_batch.expect("gpu_render_batch needs to be set for GPU mode!");
                if self.scheme.render_info.samps_per_pix % batch_size != 0 {panic!("Ensure samps_per_pix is divisble by gpu_render_batch!!")}
                let gpu_scene = GPUScene { cam: renderer_inner.scheme.cam.into(), elements: renderer_inner.scheme.scene_members.extract_concrete_types() };
                render_to_target_gpu(&self.target, &gpu_scene, || self.update_output(), &self.scheme.render_info, &iter_progress);
            }

        });
    }

    pub fn consume_and_do_anim(self, ui_mode: bool) {
        let progressbars = MultiProgress::new();
        let use_gpu = self.scheme.render_info.use_gpu.unwrap_or(false);
        let pipeline_depth: usize;
        let (region_width, region_height, render_info) = (self.scheme.render_info.width, self.scheme.render_info.height, self.scheme.render_info);

        // After extracting the locations of the each frame, render them
        let mut frame_num = 0;
        let extracted_frames = self.clone().extract_frames();
        let cam = self.scheme.cam.clone();
        println!("Extracted {} frames", extracted_frames.len());


        let frame_progress = progressbars.add(ProgressBar::new(extracted_frames.len() as u64));
        frame_progress.set_style(
            ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:80.cyan/80.blue} {pos}/{len} {msg}").unwrap()
        );
        frame_progress.enable_steady_tick(Duration::from_secs(1));
        
        
        let (frame_tx, frame_rx) = channel();

        if use_gpu { // For GPU mode, we can send the frame_tx
            
            let (done_frame_tx, done_frame_rx): (Sender<usize>, Receiver<usize>) = channel();

            // Some housekeeping before we start
            let batch_size = self.scheme.render_info.gpu_render_batch.expect("gpu_render_batch needs to be set for GPU mode!");
            if self.scheme.render_info.samps_per_pix % batch_size != 0 {panic!("Ensure samps_per_pix is divisble by gpu_render_batch!!")}
            pipeline_depth = render_info.anim_pipeline_depth.unwrap_or_else(|| {println!("No anim_pipeline_depth. defaulting to total frame count!"); extracted_frames.len()});
            println!("anim_pipeline_depth: {pipeline_depth} frames will be generated in advance");


            let scene_gen_progress = progressbars.add(ProgressBar::new(extracted_frames.len() as u64));
            scene_gen_progress.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:80.red/80.yellow} {pos}/{len} {msg}").unwrap()
            );
            scene_gen_progress.set_message("Generating scenes for each frame...");

            let iter_progress = progressbars.add(ProgressBar::new(render_info.samps_per_pix as u64));
            iter_progress.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:80.green/80.cyan} {pos}/{len} {msg}").unwrap()
            );

            let total_frame_num = extracted_frames.len();

            // Push the extracted frames asynchronously
            let mut frame_num = 0;
            thread::spawn(move || {
                let mut pending_frames = 0;
                for frame_members in extracted_frames {
                    frame_num += 1;
                    pending_frames += 1;
                    let frame_tx_inner = frame_tx.clone();

                    // Generate scene and send it to renderer.
                    frame_tx_inner.send((frame_num, GPUScene {cam: cam.into(), elements: frame_members.extract_concrete_types()})).expect("Something happened while sending the frame scene...");
                    scene_gen_progress.clone().inc(1);

                    // If we've generated more than the pipeline depths, wait until the renderer can catch up
                    while pending_frames >= pipeline_depth {
                        scene_gen_progress.set_message("Max depth reached.. Waiting for next frame to finish");
                        let done_frame_num = done_frame_rx.recv().expect("Something happened while rendering a frame...");
                        pending_frames = frame_num - done_frame_num;
                    }
                    scene_gen_progress.set_message("Generating scenes for each frame...");
                }
                scene_gen_progress.finish_with_message("Done generating each frame scene!");
                // When we're done, just loop until the rendering is done
                while done_frame_rx.recv().expect("Unexpected err") != frame_num {};
            });

            loop {
                let (frame_num, gpu_scene) = frame_rx.recv().expect("Something happened while generating frames...");

                let start = Instant::now();
                let iter_progress_inner = iter_progress.clone();
                iter_progress_inner.reset();
                let (tx, render_out) = channel();
                // Generate the frame for each
                thread::spawn(move || {
                    let region_width_inner = region_width.clone();
                    let region_height_inner = region_height.clone();
                    
                    let buf: Vec<u8> = [0, 0, 0, 0].repeat((region_width_inner * region_height_inner).try_into().unwrap());
                    let target = RenderTarget {
                        buff_mux: Arc::new(Mutex::new(buf)),
                        canv_width: region_width_inner, 
                        canv_height: region_height_inner,
                    };

                    render_to_target_gpu(&target, &gpu_scene, || tx.send(target.buff_mux.lock().clone()).expect("cannot send??"), &self.scheme.render_info, &iter_progress_inner);
                });
                
                ui_util::io_on_render_out(render_out, (region_width.clone(), region_height.clone()), ui_mode.clone(), Some(format!("anim_frames/{frame_num}.png")));
                let _ = done_frame_tx.send(frame_num);
                frame_progress.inc(1);
                frame_progress.set_message(format!("Rendering frames... Previous frame #{frame_num}: {:.3?}", start.elapsed()));
                if frame_num == total_frame_num {break;}
            }
        }
        else { // When in CPU mode, do everything sequentially

            let iter_progress = progressbars.add(ProgressBar::new(render_info.samps_per_pix as u64));
            iter_progress.set_style(
                ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:80.green/80.cyan} {pos}/{len} {msg}").unwrap()
            );

            for frame_members in extracted_frames {
                let start = Instant::now();
                let iter_progress_inner = iter_progress.clone();
                iter_progress_inner.reset();
                let (tx, render_out) = channel();

                thread::spawn(move || {                    
                    let region_width_inner = region_width.clone();
                    let region_height_inner = region_height.clone();
                    
                    let buf: Vec<u8> = [0, 0, 0, 0].repeat((region_width_inner * region_height_inner).try_into().unwrap());
                    let target = RenderTarget {
                        buff_mux: Arc::new(Mutex::new(buf)),
                        canv_width: region_width_inner, 
                        canv_height: region_height_inner,
                    };
                    
                    let skene = Scene { cam: cam.into(), members: frame_members.into() };
                    render_to_target_cpu(&target, &skene, || tx.send(target.buff_mux.lock().clone()).expect("cannot send??"), &render_info, &iter_progress_inner);
                });
                
                ui_util::io_on_render_out(render_out, (region_width.clone(), region_height.clone()), ui_mode.clone(), Some(format!("anim_frames/{frame_num}.png")));
                        
                frame_progress.inc(1);
                frame_progress.set_message(format!("Rendering frames... Previous frame {frame_num}: {:.3?}", start.elapsed()));
                frame_num += 1;
            }
        }        
        
        frame_progress.finish_with_message("Rendering all frames done!!!");
    }

    

    // Extract the locations of all scene members for each frame of the animation
    fn extract_frames<'a>(self) -> Vec<VecInto<MemberTypes>>{//Vec<Scene<'a>> {
        let mut scenes:  Vec<VecInto<MemberTypes>> = Vec::new();
        let updated_locations = self.scheme.clone();
        
        for member_frame in self.scheme.clone().scene_members.extract_anim(updated_locations.render_info.framerate.expect("Ensure the framerate is added for use_gpu!")/*, updated_locations.cam*/) {
            // println!("Extracted frame: {member_frame:?}");
            // let skene: Scene =  Scene { cam: updated_locations.clone().cam.into(), members: member_frame.into() };
            // scenes.push(skene);
            scenes.push(member_frame);
        }
        scenes
    }

    fn update_output(&self) {
        self.sender.send(self.target.buff_mux.lock().clone()).expect("cannot send??");
    }
}