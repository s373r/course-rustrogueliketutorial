# Course: Roguelike Tutorial - in Rust

Course link: https://github.com/amethyst/rustrogueliketutorial

Status: 🚧

## Preparations

1. Install WASM-related components
```shell
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```
2. `[Optional]` Install system dependencies for native target: `cmake`, `c++`, `fontconfig`
```shell
# For Fedora
sudo dnf install cmake g++ fontconfig-devel
```
3. `[Optional]` Install `simple-http-server` for file serving
```shell
cargo install simple-http-server
```

## Run

### Native
```shell
cargo run
```

### Web

Build
```shell
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/*wasm --out-dir wasm --no-modules --no-typescript
```
Serve
```shell
simple-http-server -- wasm
```
... or use Python for serving
```shell
python3 -m http.server -d wasm
```

Open http://0.0.0.0:8000/

### Index legend

- 📝 - a link to a book page
- ✏️ - a link to an `.rs` file (code)
- 👷 - a page under construction in the course
- 🚧 - not finished

## Index

- [📝 Introduction](https://bfnightly.bracketproductions.com/rustbook/chapter_0.html)
- [📝 1. Building for the Web](https://bfnightly.bracketproductions.com/rustbook/webbuild.html)
- [📝 2. Section 1 - Hello Rust](https://bfnightly.bracketproductions.com/rustbook/chapter_1.html)
  - [📝 2.1. Entities and Components](https://bfnightly.bracketproductions.com/rustbook/chapter_2.html)
  - [📝 2.2. Walking A Map](https://bfnightly.bracketproductions.com/rustbook/chapter_3.html)
  - 🚧 2.3. A More Interesting Map
  - 🚧 2.4. Field of View
  - 🚧 2.5. Monsters
  - 🚧 2.6. Dealing Damage
  - 🚧 2.7. User Interface
  - 🚧 2.8. Items and Inventory
  - 🚧 2.9. Ranged Scrolls/Targeting
  - 🚧 2.10. Saving and Loading
  - 🚧 2.11. Delving Deeper
  - 🚧 2.12. Difficulty
  - 🚧 2.13. Equipment
- 🚧 3. Section 2 - Stretch Goals
  - 🚧 3.1. Nice Walls with Bitsets
  - 🚧 3.2. Bloodstains
  - 🚧 3.3. Particle Effects
  - 🚧 3.4. Hunger Clock
  - 🚧 3.5. Magic Mapping
  - 🚧 3.6. REX Paint Menu
  - 🚧 3.7. Simple Traps
- 🚧 4. Section 3 - Generating Maps
  - 🚧 4.1. Refactor Map Building
  - 🚧 4.2. Map Building Test Harness
  - 🚧 4.3. BSP Room Dungeons
  - 🚧 4.4. BSP Interior Design
  - 🚧 4.5. Cellular Automata Maps
  - 🚧 4.6. Drunkard's Walk Maps
  - 🚧 4.7. Mazes and Labyrinths
  - 🚧 4.8. Diffusion-limited aggregation maps
  - 🚧 4.9. Add symmetry and brushes to the library
  - 🚧 4.10. Voronoi Hive Maps
  - 🚧 4.11. Wave Function Collapse
  - 🚧 4.12. Prefabs & Sectionals
  - 🚧 4.13. Room Vaults
  - 🚧 4.14. Layering/Builder Chaining
  - 🚧 4.15. Fun With Layers
  - 🚧 4.16. Room Builders
  - 🚧 4.17. Better Corridors
  - 🚧 4.18. Doors
  - 🚧 4.19. Decouple map size from screen size
  - 🚧 4.20. Section 3 Conclusion
- 🚧 5. Section 4 - Making A Game
  - 🚧 5.1. Design Document
  - 🚧 5.2. Raw Files, Data-Driven Design
  - 🚧 5.3. Data-Driven Spawn Tables
  - 🚧 5.4. Making the town
  - 🚧 5.5. Populating the town
  - 🚧 5.6. Living bystanders
  - 🚧 5.7. Game Stats
  - 🚧 5.8. Equipment
  - 🚧 5.9. User Interface
  - 🚧 5.10. Into the Woods!
  - 🚧 5.11. XP
  - 🚧 5.12. Backtracking
  - 🚧 5.13. Into the caverns
  - 🚧 5.14. Better AI
  - 🚧 5.15. Spatial Indexing Revisited
  - 🚧 5.16. Item Stats and Vendors
  - 🚧 5.17. Deep caverns
  - 🚧 5.18. Cavern to Dwarf Fort
  - 🚧 5.19. Town Portals
  - 🚧 5.20. Magic Items
  - 🚧 5.21. Effects
  - 🚧 5.22. Cursed Items
  - 🚧 5.23. Even More Items
  - 🚧 5.24. Magic Spells
  - 🚧 5.25. Enter the Dragon
  - 🚧 5.26. Mushrooms
  - 🚧 5.27. More Shrooms
  - 🚧 5.28. Ranged Combat
  - 🚧 5.29. Logging
  - 🚧 5.30. Text Layers
  - 🚧 5.31. Systems/Dispatch
  - 🚧 5.32. Dark Elf City 1
  - 🚧 5.33. Dark Elf Plaza

## Notes

### Comments

- Some of my thoughts are prefixed with `NOTE(DP):`
    - Example: `// NOTE(DP): Algorithm complexity: O(n)`
- Resolved course TODOs are prefixed with `DONE:`
    - Example: `// NOTE(DP): ^ Uncomment the above 2 lines to see the compiler error`
- Other comments copied from the course

## Code conduction

This project uses [Gitmoji](https://gitmoji.dev/) for commit messages

## License

[GPLv3+](LICENSE)
