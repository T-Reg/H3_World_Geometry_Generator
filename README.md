# H3 World Geometry Generator

Generate GLTF meshes of Uber's H3 library discrete global grid.

## What does it do?
This program walks the H3 cell hierarchy, turns every cell at a chosen *world resolution* into triangles on a sphere and exports the result as GLTF/GLB compatible data files.

The export is **chunked**: instead of one gigantic model it writes a separate `<prefix>-chunk<N>.gltf` (and matching binary `<prefix>-chunk<N>.bin`) for the children of every cell at a coarser *chunk resolution*.  
For example, with `world_res = 3` and `chunk_res = 0` you get 122 GLTF files, each containing the geometry of one base cell subdivided to resolution 3.

---

## Building
Requirements:
* Rust ≥ 1.76 (stable tool-chain)

Clone and build:
```bash
# grab the source
$ git clone <repo-url> && cd h3_world_geometry_generator

# build in debug mode
$ cargo build

# or build an optimised binary
$ cargo build --release
```

---

## Running
```bash
cargo run -- [WORLD_RES] [CHUNK_RES] [PREFIX]
```
Arguments (all optional):
1. **WORLD_RES** – H3 resolution of the geometry (default: `0`, range `0‥=15`).
2. **CHUNK_RES** – H3 resolution used to group cells into chunks (default: `0`). Must be *lower* than `WORLD_RES`.
3. **PREFIX** – Folder and filename prefix for results (default: `output`). The program creates the folder if it doesn't exist.

Example: create resolution-3 geometry, chunked at the base-cell level, using prefix `world`.
```bash
cargo run --release -- 3 0 world
```
Output tree:
```
world/
  world-chunk1.gltf
  world-chunk1.bin
  world-chunk2.gltf
  world-chunk2.bin
  …
```

During execution you will see progress such as current chunk, per-chunk percentage and overall percentage.

---

## Internals overview
* **world_gen.rs** – core logic.  `gen_world_chunks` drives the processing, triangulation, statistics and export per chunk.
* **mesh.rs** – simple deduplicating mesh container.
* **export.rs** – minimal GLTF + binary writer.
* **geometry.rs** – spherical coordinate helpers.

The code relies on:
* [`h3o`](https://crates.io/crates/h3o) for H3 indexing.
* [`rand`](https://crates.io/crates/rand`) for colour generation.
* [`serde_json`](https://crates.io/crates/serde_json`) for writing GLTF JSON.

---

## License
This project is provided "as is" under the MIT licence. Feel free to adapt for your own needs.
