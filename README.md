# Slimy Mist

This is a learning project.

You can play the game [here](https://leomeinel.github.io/slimy-mist) or download a [release](https://github.com/leomeinel/slimy-mist/releases).

## Mixed license

This repository is not entirely licensed as Apache-2.0. See [Credits](#Credits) for info on used licenses. This is especially relevant for files in `/assets`

## Packages

### Building

- [binaryen](https://archlinux.org/packages/extra/x86_64/binaryen/)

#### Android

- [cargo-ndk](https://crates.io/crates/cargo-ndk)

### Debugging

- [flamegraph](https://crates.io/crates/flamegraph)
- [mangohud](https://archlinux.org/packages/extra/x86_64/mangohud/)
- [perf](https://archlinux.org/packages/extra/x86_64/perf/)
- [wasm-server-runner](https://crates.io/crates/wasm-server-runner)
- [yq](https://archlinux.org/packages/extra/any/yq/)

### Developing

- [cargo-cache](https://crates.io/crates/cargo-cache)

## Tools

- [pixels2svg](https://github.com/ValentinFrancois/pixels2svg) for creating svgs from pixel art
- [svgo](https://github.com/svg/svgo) for minifying svgs
- [svg2vectordrawable](https://www.npmjs.com/package/svg2vectordrawable) for creating android vector drawables from svgs
- [PKGaspi](https://github.com/PKGaspi/AsepriteScripts)
- [alexpennells](https://github.com/alexpennells/AsepriteScripts)

# Credits

## Assets

| Contribution  | Author(s)                                                                                                                                                                                                                 |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Sprites       | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode) by Shave                                                                                                                                           |
| Color palette | [Free to use](https://www.reddit.com/r/gamedev/comments/qsasc4/lospeccom_color_palettes_licensing/) by [uncured-official](https://lospec.com/palette-list/uncured-official) contributors                                  |
| Music         | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode) by [freepd.com and creators](https://freepd.com/)                                                                                                  |
| SFX           | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode) by [Jaszunio15](https://freesound.org/people/Jaszunio15/packs/23837/)                                                                              |
| SFX           | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode) by OwlishMedia from [here](https://opengameart.org/content/sound-effects-pack) and [here](https://opengameart.org/content/8-bit-sound-effect-pack) |
| SFX           | [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/legalcode)/[CC-BY-3.0](https://creativecommons.org/licenses/by/3.0/legalcode) by [leohpaz](https://opengameart.org/content/12-player-movement-sfx)                |
| Fonts         | [OFL-1.1](https://opensource.org/license/OFL-1.1) by [Google Fonts](https://fonts.google.com)                                                                                                                             |

## Code/Dependencies

| Contribution                            | Author(s)                                                                                                                                                                                                                                             |
| --------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Code & Structure                        | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode)/[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_new_2d](https://github.com/TheBevyFlock/bevy_new_2d) contributors |
| Code & Game Engine                      | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy](https://crates.io/crates/bevy) contributors                                                                                             |
| Code & Asset Loading                    | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_asset_loader](https://crates.io/crates/bevy_asset_loader) contributors                                                                   |
| Code & Asset Loading/(De-)serialization | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_common_assets](https://crates.io/crates/bevy_common_assets) contributors                                                                 |
| Code & Tilemap                          | [MIT](https://opensource.org/license/MIT) by [bevy_ecs_tilemap](https://crates.io/crates/bevy_ecs_tilemap) contributors                                                                                                                               |
| Code & Input                            | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_enhanced_input](https://crates.io/crates/bevy_enhanced_input) contributors                                                               |
| Code & Particles                        | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_enoki](https://crates.io/crates/bevy_enoki) contributors                                                                                 |
| Code & Localization                     | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_fluent](https://crates.io/crates/bevy_fluent) contributors                                                                               |
| Code & Lighting                         | [MIT](https://opensource.org/license/MIT) by [bevy_lit](https://crates.io/crates/bevy_lit) contributors                                                                                                                                               |
| Code & RNG/Random                       | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_prng](https://crates.io/crates/bevy_prng) contributors                                                                                   |
| Code & RNG/Random                       | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_rand](https://crates.io/crates/bevy_rand) contributors                                                                                   |
| Code & Physics                          | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) by [bevy_rapier2d](https://crates.io/crates/bevy_rapier2d) contributors                                                                                                                     |
| Code & Game Saving                      | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_save](https://crates.io/crates/bevy_save) contributors                                                                                   |
| Code & Animations                       | [MIT](https://opensource.org/license/MIT) by [bevy_spritesheet_animation](https://crates.io/crates/bevy_spritesheet_animation) contributors                                                                                                           |
| Code & Text Input                       | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_ui_text_input](https://crates.io/crates/bevy_ui_text_input) contributors                                                                 |
| Code & Dialogue                         | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_yarnspinner](https://crates.io/crates/bevy_yarnspinner) contributors                                                                     |
| Code & Loading Progress Tracking        | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [iyes_progress](https://crates.io/crates/iyes_progress) contributors                                                                           |
| Code & Float Wrapper Types              | [MIT](https://opensource.org/license/MIT) by [ordered-float](https://crates.io/crates/ordered-float) contributors                                                                                                                                     |
| Code & Procedural Noise                 | [MIT](https://opensource.org/license/MIT) by [noisy_bevy](https://crates.io/crates/noisy_bevy) contributors                                                                                                                                           |
| Code & Navigation                       | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [polyanya](https://crates.io/crates/polyanya) contributors                                                                                     |
| Code & RNG/Random                       | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [rand](https://crates.io/crates/rand) contributors                                                                                             |
| Code & (De-)serialization               | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [serde](https://crates.io/crates/serde) contributors                                                                                           |
| Code & Tracing                          | [MIT](https://opensource.org/license/MIT) by [tracing](https://crates.io/crates/tracing) contributors                                                                                                                                                 |
| Code & Input for Mobile                 | [MIT](https://opensource.org/license/MIT) by [virtual_joystick](https://crates.io/crates/virtual_joystick) contributors                                                                                                                               |
| Code & Navigation                       | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [vleue_navigator](https://crates.io/crates/vleue_navigator) contributors                                                                       |
