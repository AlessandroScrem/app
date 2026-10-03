# Resource IDs

`ResourceId` is the common identifier used to identify resources across the engine.

It is intentionally independent from the concrete resource type. A `ResourceId` can identify an asset managed by the `AssetManager` or, when needed, a resource created directly by the runtime/GPU layer.

## Why ResourceId exists

Previously, asset identity was split between:
- a typed asset ID;
- a generation used by reusable asset slots;
- a global ID combining the asset type and asset ID.

The current model uses one monotonic `ResourceId` instead.

This gives the engine one identity primitive that can be passed between subsystems without coupling them to the `AssetManager`.

The resource type is still tracked separately by systems that need type information. `ResourceId` itself does not encode the Rust type.

## Creating an ID

Create a new ID with:

```rust
let id = ResourceId::new();
```

`ResourceId::new()` allocates a process-wide numeric ID. IDs are not reused during the lifetime of the process.

The numeric value starts at `1` for the first allocated ID. The `Default` value is `0`; it is not allocated by `ResourceId::new()` and should not be used as the identity of a real resource.

## Asset resources

Application assets normally get their ID from `AssetManager` through an `AssetHandle<T>`.

Conceptually:

```rust
let handle: AssetHandle<MaterialAsset> = assets.add(material);
let id: ResourceId = handle.id();
```

The important distinction is:
- `AssetHandle<T>` is the typed handle used by asset-oriented code.
- `ResourceId` is the untyped identity used when a subsystem only needs to identify the resource.

For example, GPU caches can use the `ResourceId` without knowing how the asset was stored by the `AssetManager`.

## Asset resources and runtime resources

`ResourceId` is **not limited to assets**.

There are two valid ways to obtain an ID:

1. An asset receives its ID from `AssetManager` through `AssetHandle<T>`.
2. A resource created directly by the runtime/GPU layer allocates its own ID with `ResourceId::new()`.

For example, a texture loaded as an application asset follows the asset path:

```rust
let handle: AssetHandle<TextureAsset> = assets.add(texture);
let id: ResourceId = handle.id();
```

A texture created only for the renderer does **not** need to become an asset:

```rust
let id = ResourceId::new();

gpu_texture_cache.insert(id, texture);
```

This is useful for resources such as renderer-generated textures, material-preview textures, shadow maps, render targets, or other GPU resources that need a stable identity while they are shared or referenced by multiple subsystems.

The important point is that `ResourceId` identifies the **resource**, while `AssetManager` determines whether that resource is managed as an **asset**.

Therefore:

- a resource can have a `ResourceId` without being an asset;
- a runtime resource does not need an `AssetHandle<T>`;
- creating a runtime resource must not require registering it in `AssetManager`;
- both asset-backed and runtime-created resources can use the same `ResourceId` space and can therefore be handled by the same GPU cache infrastructure.

GPU caches use `ResourceId` as the key regardless of where the resource came from:

```rust
// Asset-backed resource
let id = material_handle.id();

// Runtime-created resource
let id = ResourceId::new();

// Both are ordinary ResourceId values from the cache's point of view.
```

Do not add a `create_runtime()` API to `ResourceId`. If a cache eventually needs a higher-level operation that creates and registers a runtime resource, that operation belongs to the cache or resource manager, where its ownership and lifetime rules can be defined.

## ImGui textures

The ImGui texture registry uses the same ResourceId identity as the rest of the engine. It maps an engine resource to the imgui::TextureId required by the UI, without taking ownership of the underlying GPU resource.

This applies to both asset-backed and runtime-created textures:

```rust
// Asset-backed texture
let id = texture_asset.id();

// Runtime/GPU-created texture
let id = ResourceId::new();
```

A runtime texture does not need to be inserted into AssetManager just because the UI displays it. The renderer can give it a ResourceId and register that resource in the ImGui texture registry.

Only textures that the UI actually displays should be registered. In particular, an IblAsset currently exposes its source HDR texture in the IBL UI; the generated IBL cubemap, irradiance map, prefilter map and BRDF LUT are renderer resources used by rendering and are not registered unless a UI panel needs to display one of them.

The shadow-map debug texture is a runtime GPU resource. Its ResourceId is owned by the ImGui registry and its ImGui binding is created once because ShadowManager keeps the underlying RGBA texture alive for the lifetime of the runtime. Rendering new shadow contents into that texture does not require replacing the ImGui binding.

The material-preview render target follows the same runtime-resource model, but its ImGui binding is replaced when the preview render target changes.

## Converting to a number

`ResourceId::raw()` exposes the underlying `usize`:

```rust
let value: usize = id.raw();
```

Use `raw()` only when an API genuinely requires a numeric value, for example:
- logging/debug output;
- serialization formats that explicitly require a number;
- indexing into a structure designed around numeric IDs.

Do not use the raw value as the primary resource identity. Pass `ResourceId` itself between engine subsystems.

There is intentionally no `from_raw()` constructor. Reconstructing an arbitrary ID would bypass the allocator and could create collisions with IDs already allocated by the process.

## Lifetime and reuse

`ResourceId` is an identity, not a storage slot.

Removing an asset does not make its numeric ID available for reuse. A later resource receives a new ID.

This is important because an old `ResourceId` cannot silently start referring to a different resource after deletion.

Storage ownership and resource lifetime are separate concerns:
- `ResourceId` identifies a resource;
- `AssetManager` owns asset storage and bookkeeping;
- GPU caches own their GPU representations;
- dependency/reference systems decide when a resource can be removed.

## What not to do

Do not:
- generate IDs manually with integers;
- reconstruct IDs from `raw()` values;
- reuse an old `ResourceId` for a new resource;
- add `create_runtime()` wrappers without a concrete use case;
- make `ResourceId` responsible for knowing the resource type.

Prefer:

```rust
let id = ResourceId::new();
```

and pass the `ResourceId` unchanged between the systems that need to identify that resource.

## Typed IDs

Asset-specific aliases such as `MeshId`, `MaterialId`, `TextureId` and `IblId` currently map to `ResourceId`.

They are useful as domain terminology while keeping the underlying identity representation unified.

When code needs the type-safe asset handle and access to the asset itself, use `AssetHandle<T>`. When code only needs to identify a resource, use `ResourceId`.

## Design rule

The rule of thumb is:

> Use `AssetHandle<T>` when you need an asset with a known type. Use `ResourceId` when you need to identify a resource across subsystems.

The ID should remain deliberately small and boring. Resource lookup, ownership, dependencies and GPU lifetime belong to the systems that manage those concerns, not to `ResourceId` itself.