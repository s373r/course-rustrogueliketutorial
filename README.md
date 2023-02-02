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
  - [📝 2.3. A More Interesting Map](https://bfnightly.bracketproductions.com/rustbook/chapter_4.html)
  - [📝 2.4. Field of View](https://bfnightly.bracketproductions.com/rustbook/chapter_5.html)
  - [📝 2.5. Monsters](https://bfnightly.bracketproductions.com/rustbook/chapter_6.html)
  - [📝 2.6. Dealing Damage](https://bfnightly.bracketproductions.com/rustbook/chapter_7.html)
  - [📝 2.7. User Interface](https://bfnightly.bracketproductions.com/rustbook/chapter_8.html)
  - [📝 2.8. Items and Inventory](https://bfnightly.bracketproductions.com/rustbook/chapter_9.html)
  - [📝 2.9. Ranged Scrolls/Targeting](https://bfnightly.bracketproductions.com/rustbook/chapter_10.html)
  - [📝 2.10. Saving and Loading](https://bfnightly.bracketproductions.com/rustbook/chapter_11.html)
  - [📝 2.11. Delving Deeper](https://bfnightly.bracketproductions.com/rustbook/chapter_12.html)
  - [📝 2.12. Difficulty](https://bfnightly.bracketproductions.com/rustbook/chapter_13.html)
  - [📝 2.13. Equipment](https://bfnightly.bracketproductions.com/rustbook/chapter_14.html)
- [📝 3. Section 2 - Stretch Goals](https://bfnightly.bracketproductions.com/rustbook/chapter_15.html)
  - [📝 3.1. Nice Walls with Bitsets](https://bfnightly.bracketproductions.com/rustbook/chapter_16.html)
  - [📝 3.2. Bloodstains](https://bfnightly.bracketproductions.com/rustbook/chapter_17.html)
  - [📝 3.3. Particle Effects](https://bfnightly.bracketproductions.com/rustbook/chapter_18.html)
  - [📝 3.4. Hunger Clock](https://bfnightly.bracketproductions.com/rustbook/chapter_19.html)
  - [📝 3.5. Magic Mapping](https://bfnightly.bracketproductions.com/rustbook/chapter_20.html)
  - [📝 3.6. REX Paint Menu](https://bfnightly.bracketproductions.com/rustbook/chapter_21.html)
  - [📝 3.7. Simple Traps](https://bfnightly.bracketproductions.com/rustbook/chapter_22.html)
- [📝 4. Section 3 - Generating Maps](https://bfnightly.bracketproductions.com/rustbook/chapter23-prefix.html)
  - [📝 4.1. Refactor Map Building](https://bfnightly.bracketproductions.com/rustbook/chapter_23.html)
  - [📝 4.2. Map Building Test Harness](https://bfnightly.bracketproductions.com/rustbook/chapter_24.html)
  - [📝 4.3. BSP Room Dungeons](https://bfnightly.bracketproductions.com/rustbook/chapter_25.html)
  - [📝 4.4. BSP Interior Design](https://bfnightly.bracketproductions.com/rustbook/chapter_26.html)
  - [📝 4.5. Cellular Automata Maps](https://bfnightly.bracketproductions.com/rustbook/chapter_27.html)
  - [📝 4.6. Drunkard's Walk Maps](https://bfnightly.bracketproductions.com/rustbook/chapter_28.html)
  - [📝 4.7. Mazes and Labyrinths](https://bfnightly.bracketproductions.com/rustbook/chapter_29.html)
  - [📝 4.8. Diffusion-limited aggregation maps](https://bfnightly.bracketproductions.com/rustbook/chapter_30.html)
  - [📝 4.9. Add symmetry and brushes to the library](https://bfnightly.bracketproductions.com/rustbook/chapter_31.html)
  - [📝 4.10. Voronoi Hive Maps](https://bfnightly.bracketproductions.com/rustbook/chapter_32.html)
  - [📝 4.11. Wave Function Collapse](https://bfnightly.bracketproductions.com/rustbook/chapter_33.html)
  - [📝 4.12. Prefabs & Sectionals](https://bfnightly.bracketproductions.com/rustbook/chapter_34.html)
  - [📝 4.13. Room Vaults](https://bfnightly.bracketproductions.com/rustbook/chapter_35.html)
  - [📝 4.14. Layering/Builder Chaining](https://bfnightly.bracketproductions.com/rustbook/chapter_36.html)
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
