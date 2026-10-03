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

## GPU caches

GPU caches use `ResourceId` as the key for resources associated with assets.

For example, the material/mesh/texture cache can associate a GPU resource with:

```rust
let id = material_handle.id();

// Conceptually:
// gpu_material_cache[id] -> GPU material
```

This keeps the GPU layer independent from the internal representation of asset storage.

A GPU resource that is not backed by an asset can also receive its own ID:

```rust
let id = ResourceId::new();
```

Only introduce such an ID when the runtime resource actually needs to be identified across systems. Do not create runtime-specific wrapper functions merely to hide `ResourceId::new()`.

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