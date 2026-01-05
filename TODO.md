

App::init()
 ├─ World::new()
 ├─ Resources::default()
 │    └─ insert(RenderQueue)
 │    └─ insert(UiRenderQueue)
 │    └─ insert(BoundingBoxQueue)
 │    └─ insert(LightQueue)
 └─ Renderer::new()


ECS 
    - imgui_flush_selected
    - imgui_create (component_view)
    - input_update
    - apply_commands (domain commands)
    - camera_orbit
    - picking
    - update_hieararchy_system 
    - update_bounding_box_system
    - recenter_camera_system
 
 GPU update

    +- create (frame view and encoder)
    +- imgui_update (platform context) every frame
    +- update_global_uniform_to_gpu
    +- update_light_uniform_to_gpu
    +- update_bounding_box_to_gpu // require hierarchy update
    +- update_model_uniforms_to_gpu
    +- update_material_system_to_gpu

    RenderPasses:

        +- render_mesh
        +- render_light
        +- render_skybox
        +- render_axis
        +- render_bounding_box
        +- render_hdr_to_ldr
        +- render_outline
        - read_entity_id_to_buffer
        +- render_imgui

    - submit encoder and present frame



REDERER create:
    +    (adapter);
    +    (device);
    +    (queue);
    +    (surface);
    +    (surface_config);
    +    (texture_manager);
    +    (gpu_manager);
    +    (pipeline_manager);
    +    (material_manager);
    +    (light_manager);
    +    (mesh_manager);
    +    (bbox_manager);
        (pickobject);
    +    (skybox_manager);
    +    (imgui); 