# App
![Linux](https://github.com/<utente>/<repo>/actions/workflows/rust-linux.yml/badge.svg)
![macOS](https://github.com/<utente>/<repo>/actions/workflows/rust-macos.yml/badge.svg)
![Windows](https://github.com/<utente>/<repo>/actions/workflows/rust-windows.yml/badge.svg)

# Simple Renderer Wgpu 

A modern 3D/2D render engine that uses wgpu

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
- [ ] Entity selection
- [ ] Outline selection
- [ ] Hierarchy entity
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
cargo test
cargo run --release
```


## Screenshots
![Hello PBR Cube](/assets/screenshots/Screenshot_2025-09-27.jpg?raw=true "Hello PBR cube!")

## Known issues
| Fixed    | Prioriry |              Description                                                   |
| :---:    | :---:    | :---                                                                       |
|`fix`     |   Low    | WGSL std140 / std430 uniform/storage buffer require allignemets to 16 bytes|    

