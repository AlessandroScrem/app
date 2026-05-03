# App
[![Rust Linux](https://github.com/AlessandroScrem/app/actions/workflows/rust-linux.yml/badge.svg)](https://github.com/AlessandroScrem/app/actions/workflows/rust-linux.yml)
[![Rust Windows](https://github.com/AlessandroScrem/app/actions/workflows/rust-windows.yml/badge.svg)](https://github.com/AlessandroScrem/app/actions/workflows/rust-windows.yml)
[![Rust macOS](https://github.com/AlessandroScrem/app/actions/workflows/rust-macos.yml/badge.svg)](https://github.com/AlessandroScrem/app/actions/workflows/rust-macos.yml)

# Simple Renderer Wgpu 

A modern 3D/2D render engine that uses wgpu

```toml
[dependencies]
wgpu = "29"
imgui-wgpu = "0.28.0"
imgui = "0.12"
winit = "0.30.9"
legion = {version = "0.4.0", features = ["extended-tuple-impls"]}
cgmath = "0.18.0"
```


## Features

- [x] Ecs 
- [x] Phong shading
- [x] Imgui
- [x] Mesh entity
- [x] Light entity
- [x] SkyBox
- [x] IBL
- [x] PBR
- [x] Exposure
- [x] Tonemapping
- [x] Skybox selctor
- [x] xyz Axis
- [x] Docking Gui
- [x] Mesh Bounding box
- [x] Entity selection
- [x] Outline selection
- [x] Hierarchy entity
- [x] Import gltf with hierarchy
- [x] Material Editor per Submesh
- [x] Material gltf Transmission
- [x] Material gltf Volume
- [x] Material gltf Ior
- [x] Material gltf TextureTransform
- [x] Material gltf TextureCoords[0,1]
- [x] Environment rotation
- [ ] Direct Light shadow
- [ ] SSAO   
- [ ] 

## Platforms
 - [x] Windows
 - [ ] Linux
 - [ ] MacOs

## Compiling and running

```bash
git clone https://github.com/AlessandroScrem/app.git
cd app

cargo build
cargo test --lib
cargo run --release -- -v

// run with options 
cargo run --release -- --help
cargo run --release -- -w<WIDTH> -h<HEIGHT> --verbose
```


## Screenshots
![Hello PBR Cube](/assets/screenshots/hello_cube-2025-09-27.jpg?raw=true "Hello PBR cube!")

## Known issues
| Fixed    | Prioriry |              Description                                                   |
| :---:    | :---:    | :---                                                                       |
| `todo`   |   Low    | Background color is influenced by final filtering (gamma,  color filtering)|    
| `todo`   |   Low    | Environment / Skybox orientation different fromGltfViewer -90 Y            |    
| `fixed`  |   High   | Panic if click on bbox ui if no meshes in scene                            |    
| `fixed`  |   Low    | WGSL std140 / std430 uniform/storage buffer require allignemets to 16 bytes|    

