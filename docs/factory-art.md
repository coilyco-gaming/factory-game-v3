# Factory runtime art

The local Bevy viewer loads a small accepted sprite set without turning the
retained Unity library into a runtime dependency. The typed `FactoryArt`
resource owns every image handle under the shell crate's default `assets`
root.

## Accepted files

- `factory/terrain/ground.png`
- `factory/logistics/road-straight-ns.png`
- `factory/vehicles/truck.png`
- `factory/resources/iron-ore-deposit.png`
- `factory/resources/copper-ore-deposit.png`
- `factory/resources/coal-deposit.png`
- `factory/resources/stone-deposit.png`
- `factory/machines/foundry.png`
- `factory/machines/factory.png`
- `factory/machines/coal-plant.png`
- `factory/machines/radar.png`
- `factory/machines/mining-drill.png`
- `factory/structures/warehouse.png`
- `factory/items/iron-ore.png`
- `factory/items/iron-bars.png`

All fifteen files are 100x100 RGBA images under
`crates/factory_shell/assets/factory/`. The narrow `.gitattributes` exception
keeps only that subtree in ordinary Git. The retained Unity image library
remains LFS-managed and inert.

## Projection rules

The ground image stretches across the complete world in one sprite. Iron,
copper, coal, and stone sources receive matching deposit overlays. A source
activated by a mining-drill radar receives the drill overlay while deployed.
Iron-bar factories receive the foundry, while other factories use the general
factory. Coal-plant generators, radars, and completed storage warehouses use
their matching sprites. Every hauler receives the truck. Iron ore and iron
bars in authoritative cargo select the matching item icon.

The north-south road sprite is rotated 90 degrees only for a road with east
and west neighbors and no north or south neighbors. North-south straights use
the source orientation. Corners, junctions, endpoints, and co-located nodes
retain the colored road fallback.

Colored nodes, haulers, and cargo badges remain underneath the art. Missing
images therefore degrade to the previous presentation. Labels, gauges, route
dashes, frame-time effects, detail visibility, and hauler interpolation keep
their existing ownership and behavior.

## Packaging boundary

Native development reads these files directly from the crate asset root. The Wasm target
compiles with the same handles, but this change does not alter Trunk output,
the container image, or deployment. Browser delivery and parity verification
are tracked in [issue #60](https://forgejo.coilysiren.me/coilyco-gaming/factory-game-v3/issues/60).
