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
The top status bar reuses the four deposit sprites as 18px resource icons next
to authoritative iron, copper, coal, and stone stockpile counts. The icon
entities stay fixed while only their adjacent count text updates.

The north-south road sprite is rotated 90 degrees only for a road with east
and west neighbors and no north or south neighbors. North-south straights use
the source orientation. Corners, junctions, endpoints, and co-located nodes
retain the colored road fallback.

Colored node fallbacks render only for identities without accepted art. Real
node sprites retain activity pulses without the old rectangular backplates,
and truck art replaces the colored hauler underlay. Cargo badges, labels,
gauges, route dashes, detail visibility, and interpolation retain their state
and animation roles.

## Packaging boundary

Native development reads these files directly from the crate asset root. The
browser build reaches the same files over HTTP: Bevy's `AssetServer` requests
`/assets/<path>` in the browser, so `index.html` carries a Trunk `copy-dir`
link that places the crate asset root at `dist/assets`. The container image
copies the whole `dist` tree, so no separate Dockerfile step is needed.

Sprite names carry no content hash, because the paths are compiled into the
binary. `nginx.conf` therefore serves `/assets/` with a one-hour revalidated
cache instead of the immutable policy the hashed Trunk bundles use, and with
`try_files ... =404` so a missing sprite fails as a real 404. Without that the
SPA fallback would answer an image request with `index.html`, masking the miss.
On a failed request the viewer keeps the same colored fallback it uses
natively for unsprited identities.
