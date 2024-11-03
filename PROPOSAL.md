<!-- omit in toc -->
# Project Proposal

Project proposal for ECE1724 - Performant Software Systems with Rust

Team members: Jackson Nie (1005282409) Jun Ho Sung ()

<!-- omit in toc -->
## Table of Contents
- [Motivation](#motivation)
    -[Testing](#testing)
- [Objectives](#objectives)
  - [Primary Objectives](#primary-objectives)
  - [Secondary Objectives](#secondary-objectives)
- [Key features](#key-features)
  - [Primary Features](#primary-features)
  - [Secondary Features](#secondary-features)
- [Tentative plan](#tentative-plan)

## Motivation
Our project proposal stems from a shared passion for gaming and computer graphics, combined with a fascinating opportunity to enhance an existing ray tracing implementation.

Ray tracing, which creates photorealistic images by simulating light-object interactions, has become increasingly relevant in modern graphics applications, especially in gaming and visual effects. While the current ray-tracer implementation produces high-quality images through multithreading and kd-tree acceleration, its performance remains a significant bottleneck. This limitation presents an exciting opportunity for optimization, particularly through GPU acceleration. 

This motivated us to propose an endeavor that aims to accelerate the ray-tracing process leveraging modern GPUs. Both team members have experience in GPU programming with C++, but neither has experience implementing GPU solutions in Rust. This project therefore offers an ideal intersection of learning and practical application. As a language that promises "blazingly fast" performance while maintaining memory safety, Rust presents an excellent platform for high-performance computing tasks like ray tracing. By accelerating a computationally intensive graphics application, we will gain hands-on experience and learn the techniques of writing high-performance applications in Rust. This aligns perfectly with the course's goal of developing performant and scalable systems.

Beyond pure performance optimization, we aim to expand the ray tracer's capabilities by implementing new features such as an interactive, visual user interface as well as short scene generation. These additions will make the project more engaging, and this combination of optimization and feature development will also provide additional challenges in maintaining performance at scale. 

This project currently has one major algorithmic optimization in the form of K-D trees. By subdividing the bounding box of objects within a scene, the program is able minimize ray tracing in areas that can be confirmed to not have any interecting objects. However, from our testing, we found that increasing the K-D tree depth can lead to longer rendering times, likely due to the size of the tree exploding as the depth increases. This means that even though the bounding box volume of the objects within the scene decrease, the time required to access all the nodes of the tree will increase. Therefore, we wanted to explore ways to further optimize this project in different ways, by accessing more hardware available to the system (GPUs), and implementing common software optimization techniqes.

We also wanted to take this project one step further. Of course, the natural step to make is to render animations. By leveraging the significant speedup we plan on achieving, we hope to create a scalable video rendering program.

We want to add a usable UI as well, to make it easier to use for anyone who wants to try rendering a scene of their own.

To get a baseline performance profile of the project at its current state, we rendered two different scenes with varying parameters. 
We used one Windows 11 and one MacOS machine to help see how different systems perform. By the end of the project, we will re-run the same tests after the project to determine what speedup we were able to achieve.

During the test runs, we noticed that the image turned out grainy a lot of the time with low-iteration or low-depth parameters. This was due to the renders being purely ray-traced, meaning that every light 'rays' generated had to hit every single point in the object to create a smooth render, making it highly compute intensive. Due to this effect, we were determined to find any ways that could make the quality of the image higher while keeping the number of computations the same.
To summarize, through this project, we expect to:
* Significantly improve ray tracing performance through GPU acceleration
* Gain practical experience with Rust in high-performance computing
* Implement new features that showcase intriguing capabilities of the ray tracer

### Testing
* Parameters to sweep
1. samps_per_pix: 100 - 1000 - 10000
2. assured_depth: 1   - 2    - 5
3. kd_tree_depth: 2   - 8    - 17
    
Jun Ho's machine:
* CPU: AMD Ryzen 5800x
* GPU: AMD Radeon RX6800XT
* RAM: 32GB 3200MHz
* OS: Windows 11

Test
1. Wada
    1. wada_100_1_2: 21.9s
    2. wada_100_1_8: 27.6s
    3. wada_100_1_17: 28.4s
    4. wada_100_2_2: 25.7s
    5. wada_100_5_2: 41.0s
    6. wada_1000_1_2: 191.2s
    7. wada_10000_1_2: 1928.6s
    8. wada_1000_2_8: 372.6s
    9. wada_10000_5_17: 6914.1s

2. Biplane
    1. biplane_100_1_2: DNF (too long per iteration - ~885s/it)
    2. biplane_100_1_8: 2316.1s
    3. biplane_100_1_17: 328.3s
    4. biplane_100_2_2: DNF (too long per iteration - ~995s/it)
    5. biplane_100_5_2: DNF (too long per iteration - ~s/it)
    6. biplane_1000_1_2: DNF (too long per iteration - ~885s/it)
    7. biplane_10000_1_2: DNF
    8. biplane_1000_2_8: 
    9. biplane_10000_5_17: 37628.7s

3. 

Jackson's machine:
    CPU: (jackson to add)
    GPU:
    RAM:
    OS:
Test
1. 
2. 
3. 
//// End TODO

## Objectives
### Primary Objectives
1. Performance Optimization
   * Port the existing CPU-based ray tracer to utilize GPU acceleration in RUst.
   * Implement parallel processing algorithms for various GPU architectures. Develop a flexible GPU backend that supports multiple architectures through generic GPU computation crates.
   * Explore NVIDIA CUDA-specific implementation. 
   * Conduct thorough performance analysis to:
     * Identify computational bottlenecks in the current implementation.
     * Determine optimal GPU-accelerated algorithms for ray tracing operations.
       * Since the bulk of the operations are simple math applied to a wide range of pixels, we believe that GPU acceleration fits this problem extremely well.
   * Based on the test results, target a minimum 5x speedup over current CPU implementation.
   * Implement comprehensive benchmarking suite to:
     * Compare CPU vs. GPU performance metrics.
     * Identify potential drawbacks/limitations of the CPU/GPU implementations, and analyse their impact on performance.
       * Document these optimization impacts and trade-offs.

2. Interactive User Interface
   * The current implementation utilizes the command line for generating ray traces.
   * Implement an interactive user interface that allows the user to preview images to trace, along with interactive hyperparameter toggling capabilities.

3. Animation System Integration
   * Develop a kinematic-based animation system.
   * Implement efficient frame generation pipeline leveraging GPU accelerated algorithms implemented previously.


### Secondary Objectives
1. Animation System Optimization
   * Analyze bottlenecks/slowdowns in the animation pipeline.
   * Identify potential improvements that can be applied to speed up the process.
   * Integrate the animation generation process into the UI to provide real-time render previews.

2. Pre-Rasterization Enhancement
   * Design and implement a GPU-accelerated pre-rasterization pass.
   * Integrate rasterization output with the ray tracing system.
   * Evaluate the performance impact of pre-rasterization.
   * Analyse image quality with/without pre-rasterization with the same number of iterations.

## Key features
### Primary Features
1. Performance Optimization
   * Multi-architecture GPU Backend
     * Generic gpu backend that utilizes existing crates to run on different gpu architectures.
     * Rhe utilization of the [emu](https://github.com/calebwin/emu) crate will be our first experiment, because emu provides a clean macro interface for running programs on the GPU and has a relatively simple programming pattern.
     * Alternatively, implementation will explore wgpu for cross-platform compatibility [wgpu](https://github.com/gfx-rs/wgpu).
   * Nvidia-specific CUDA implementation
     * Targets Nvidia GPUs specifically.
     * Currently aiming to utilize the [Rust-CUDA](https://github.com/Rust-GPU/Rust-CUDA/tree/master) project for implementing CUDA kernels.
   * CPU Performance Enhancement
     * Further optimization of existing multi-threaded CPU implementation by identifying sub-optimal execution patterns and optimizable performance bottlenecks.

Ray intersection calculations can be sent off to the GPU for ultra-quick processing.

The following crates will be explored to add GPU acceleration:
* emu - procedural macro uses OpenCL to automatically determine what parts of the provided code can be accelerated with a GPU. As it's automatic, it will be the easiest to use, but will likely not provide as much optimization compared to the below two options.
* WebGPU - Will need to compartmentalize the parallelizable code into compute shaders to then provide as a shader module for the program.
* opencl3

2. Interactive User Interface
   * Real-time render and preview capabilities.
     * Preview functionality for image selection, and interactive selection opposed to command-line argument input.
   * Configuration of render settings and parameter adjustment.
     * Visual controls for render setting and parameters.
     * Real-time parameter adjustment without configuration file editing.
     * Intuitive preset management.
   * Performance monitoring
     * Live progress tracking and estimated completion time.
     * Detailed performance metrics denoting time spent in exhaustive regions of calculation and bottlenecks. 
     * Can be interactively toggled on/off.

3. Animation System Integration
   * Physics-driven object motion.
   * Smooth camera path interpolation and rotation of object.    
    1. Keyframe
    2. cv::videoio::VideoWriter
    1. Camera movement
    2. Object movement

### Secondary Features
1. Animation System Optimization
   * UI Integration and Monitoring
     * Real-time animation parameter controls in the interactive interface.
     * Live performance metrics and profiling data.
     * Frame-by-frame preview and adjustment capabilities.
   * Performance Enhancement
     * Systematic analysis of animation pipeline bottlenecks.
     * Implementation of caching strategies for repeated calculations.
2. Pre-Rasterization Enhancement
   * Quality Improvement Pipeline
     * Implementation of GPU-accelerated pre-rasterization stage.
     * Integration with existing ray tracing pipeline.
     * This will allow the render to generate a clear, serviceable image with much less compute required than a purely ray-traced render. This will produce a high-quality render in a shorter time by taking the best of both of rasterization (fast compute for a complete image) and ray tracing (realistic reflections). 


## Tentative plan

Submission deadline: Monday December 16th (6 weeks from proposal due)
To talk about in the weekend
* Week 1 (11/04 - 11/10) - 
* Week 2 (11/11 - 11/17) - 
* Week 3 (11/18 - 11/24) - 
* Week 4 (11/25 - 12/01) - 
* Week 5 (12/02 - 12/08) - 
* Week 6 (12/09 - 12/15) - 