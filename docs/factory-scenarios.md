# Factory simulation scenarios

Twelve deterministic layouts, all of them headless. **None is player-facing.**
The viewer runs the compact planning game in
[compact-first-playable.md](compact-first-playable.md), which is not in this
catalog, so a layout here is a fixture rather than a world someone plays.

That matters when reading a long run. Most of these assert one mechanism and
then have nothing left to do, and a fixture that has finished looks exactly
like a world that is stuck. See [stranded-products.md](stranded-products.md)
for the field that separates them.

- **Iron bars** - one hauler supplies one foundry from a finite iron deposit.
- **Iron bars fleet** - three haulers arbitrate the same bounded demand.
- **Building materials** - two source types satisfy a multi-input recipe.
- **Powered ironworks** - coal logistics charge an energy-gated factory grid.
- **Drill deployment** - a hauler retrieves a spawnable drill, deploys it at a
  radar-claimed dormant source, and drains the source before teardown.
- **Obstacle convoy** - two haulers route around an occupied cell and arbitrate
  a shared transit cell.
- **Drill production chain** - adjacent inserters convert iron and copper ore
  through five indexed factories into mining drills.
- **Distributed frame line** - separated factories force haulers to discover
  foundry output, retrieve it from an adjacent cell, cross the grid, and satisfy
  downstream frame demand.
- **Automatic grid link** - a distant coal plant greedily constructs three
  battery-backed line cells that energize an ironworks network.
- **Warehouse construction** - a factory advertises spawnable warehouse
  inventory, a hauler retrieves and deploys it, and the build site becomes an
  occupied structure.
- **Hybrid generator grid** - fueled and fuel-free generators produce together,
  build independent links, and balance their combined output across the grid.
- **V3 50x50 factory world** - 423 generated ore deposits, one manifest stone
  source, three mining-drill radars, one coal-plant radar, 15 haulers, seven
  foundries, eight factories, and the central coal plant run deployment,
  mining, freight, production, and remote power expansion together. A 650-tick
  release proof survives a complete non-generator energy cutoff at tick 500.

Every layout is reachable through the headless CLI and the test suite, and
nowhere else. The 50x50 world is not an exception to that: it is migration
evidence at full scale rather than playable content, which
[v3-worlds.md](v3-worlds.md) records along with its provenance.

The eleven small layouts carry the sim's test coverage, `iron-bars` alone
standing behind roughly two dozen assertions and the CLI's default
`--scenario`. Three authored 50x50 worlds that carried none were removed in
`teable:coilyco-gaming/factory-game-v3#7041`.
