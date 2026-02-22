# Slimy Mist

This is a learning project.

You can play the game [here](https://leomeinel.github.io/slimy-mist) or download a [release](https://github.com/leomeinel/slimy-mist/releases).

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

## Mixed license

This repository is not entirely licensed as Apache-2.0. More details about the author(s) can be found in [Credits](#credits)

| Files                                                                                                                 | Author(s)                                                | License                                                                                                                               |
| --------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `/assets/audio/music/{bar-brawl.ogg,bit-bit-loop.ogg,screen-saver.ogg}`                                               | [freepd.com & Creators](https://freepd.com/)             | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode)                                                                |
| `/assets/audio/sound-effects/impact/{damage-hit.ogg,damage-ouch.ogg,lose-wobbledown.ogg}`                             | [OwlishMedia](https://opengameart.org/users/owlishmedia) | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode)                                                                |
| `/assets/audio/sound-effects/movement/{bounce.ogg,player-walk-hard0.ogg,player-walk-hard1.ogg,player-walk-hard2.ogg}` | [OwlishMedia](https://opengameart.org/users/owlishmedia) | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode)                                                                |
| `/assets/audio/sound-effects/movement/player-jump.ogg`                                                                | [leohpaz](https://opengameart.org/users/leohpaz)         | [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/legalcode)/[CC-BY-3.0](https://creativecommons.org/licenses/by/3.0/legalcode) |
| `/assets/audio/sound-effects/ui/{click.ogg,hover.ogg}`                                                                | [Jaszunio15](https://freesound.org/people/Jaszunio15/)   | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode)                                                                |
| `/assets/data/*`                                                                                                      | [Leopold Meinel](https://github.com/leomeinel)           | [CC-BY-NC-SA-4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode)                                                        |
| `/assets/fonts/Pixeloid/*`                                                                                            | [GGBotNet](https://www.ggbot.net/)                       | [OFL-1.1](https://opensource.org/license/OFL-1.1)                                                                                     |
| `/assets/images/*`                                                                                                    | [Leopold Meinel](https://github.com/leomeinel), Shave    | [CC-BY-NC-SA-4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode)                                                        |
| `/graphics/aseprite-scripts/export_layers.lua`                                                                        | [PKGaspi](https://github.com/PKGaspi)                    | [MIT](https://opensource.org/license/MIT)                                                                                             |
| `/graphics/aseprite-scripts/image_border.lua`                                                                         | [alexpennells](https://github.com/alexpennells)          | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)                                                                             |
| `/graphics/characters/*`                                                                                              | Shave                                                    | [CC-BY-NC-SA-4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode)                                                        |
| `/graphics/levels/*`                                                                                                  | Shave                                                    | [CC-BY-NC-SA-4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode)                                                        |
| `/graphics/palettes/otterisk-96.gpl`                                                                                  | [Otterisk](https://lospec.com/otterisk)                  | N/A                                                                                                                                   |
| `/graphics/ui/*`                                                                                                      | [Leopold Meinel](https://github.com/leomeinel)           | [CC-BY-NC-SA-4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/legalcode)                                                        |

## Credits

### Created by

| Contribution              | Author(s)                                      |
| ------------------------- | ---------------------------------------------- |
| Game Design & Programming | [Leopold Meinel](https://github.com/leomeinel) |
| Sprites                   | Shave                                          |

### Assets

| Contribution  | Author(s)                                                | Source(s)                                                                                                             |
| ------------- | -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Color Palette | [Otterisk](https://lospec.com/otterisk)                  | [0](https://lospec.com/palette-list/otterisk-96)                                                                      |
| Fonts         | [GGBotNet](https://www.ggbot.net/)                       | [0](https://ggbot.itch.io/pixeloid-font)                                                                              |
| Music         | [freepd.com & Creators](https://freepd.com/)             | [0](https://freepd.com/)                                                                                              |
| SFX           | [Jaszunio15](https://freesound.org/people/Jaszunio15/)   | [0](https://freesound.org/people/Jaszunio15/packs/23837/)                                                             |
| SFX           | [leohpaz](https://opengameart.org/users/leohpaz)         | [0](https://opengameart.org/content/12-player-movement-sfx)                                                           |
| SFX           | [OwlishMedia](https://opengameart.org/users/owlishmedia) | [0](https://opengameart.org/content/sound-effects-pack), [1](https://opengameart.org/content/8-bit-sound-effect-pack) |

### Code/Dependencies

| Contribution             | Author(s)                                                                                                                                                                                                                                             |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| (De-)serialization       | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [serde](https://crates.io/crates/serde) contributors                                                                                           |
| Animations               | [MIT](https://opensource.org/license/MIT) by [bevy_spritesheet_animation](https://crates.io/crates/bevy_spritesheet_animation) contributors                                                                                                           |
| Asset (De-)serialization | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_common_assets](https://crates.io/crates/bevy_common_assets) contributors                                                                 |
| Asset Loading            | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_asset_loader](https://crates.io/crates/bevy_asset_loader) contributors                                                                   |
| Debug UI                 | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui) contributors                                                               |
| Dialogue                 | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_yarnspinner](https://crates.io/crates/bevy_yarnspinner) contributors                                                                     |
| Float Wrapper Types      | [MIT](https://opensource.org/license/MIT) by [ordered-float](https://crates.io/crates/ordered-float) contributors                                                                                                                                     |
| Game Engine              | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy](https://crates.io/crates/bevy) contributors                                                                                             |
| Input                    | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_enhanced_input](https://crates.io/crates/bevy_enhanced_input) contributors                                                               |
| Lighting                 | [MIT](https://opensource.org/license/MIT) by [bevy_lit](https://crates.io/crates/bevy_lit) contributors                                                                                                                                               |
| Loading Progress         | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [iyes_progress](https://crates.io/crates/iyes_progress) contributors                                                                           |
| Localization             | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_fluent](https://crates.io/crates/bevy_fluent) contributors                                                                               |
| Mobile Joystick          | [MIT](https://opensource.org/license/MIT) by [virtual_joystick](https://crates.io/crates/virtual_joystick) contributors                                                                                                                               |
| Navigation               | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [vleue_navigator](https://crates.io/crates/vleue_navigator) contributors                                                                       |
| Particles                | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_enoki](https://crates.io/crates/bevy_enoki) contributors                                                                                 |
| Pathfinding              | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [polyanya](https://crates.io/crates/polyanya) contributors                                                                                     |
| Physics                  | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0) by [bevy_rapier2d](https://crates.io/crates/bevy_rapier2d) contributors                                                                                                                     |
| Procedural Noise         | [MIT](https://opensource.org/license/MIT) by [noisy_bevy](https://crates.io/crates/noisy_bevy) contributors                                                                                                                                           |
| Project Structure        | [CC0-1.0](https://creativecommons.org/publicdomain/zero/1.0/legalcode)/[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_new_2d](https://github.com/TheBevyFlock/bevy_new_2d) contributors |
| RNG                      | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_prng](https://crates.io/crates/bevy_prng) contributors                                                                                   |
| RNG                      | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_rand](https://crates.io/crates/bevy_rand) contributors                                                                                   |
| RNG                      | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [rand](https://crates.io/crates/rand) contributors                                                                                             |
| Save States              | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_save](https://crates.io/crates/bevy_save) contributors                                                                                   |
| Text Input               | [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)/[MIT](https://opensource.org/license/MIT) by [bevy_ui_text_input](https://crates.io/crates/bevy_ui_text_input) contributors                                                                 |
| Tilemap                  | [MIT](https://opensource.org/license/MIT) by [bevy_ecs_tilemap](https://crates.io/crates/bevy_ecs_tilemap) contributors                                                                                                                               |
| Tracing                  | [MIT](https://opensource.org/license/MIT) by [tracing](https://crates.io/crates/tracing) contributors                                                                                                                                                 |
