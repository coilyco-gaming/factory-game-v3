# Factory simulation scenarios

The deterministic scenario catalog covers fifteen layouts:

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
- **Legacy assembly yard** - the early C# row yard places a coal plant, three
  foundries, seven assembly factories, and eight haulers between three active
  authored ore fields.
- **Twin plant basin** - the later C# two-generator yard adds three deployment
  radars, two coal plants, six factories, and twelve haulers around three
  dormant authored ore fields.
- **Four corners works** - a new distributed yard separates iron and copper
  foundries across the center, places coal at the north and south edges, and
  gives sixteen haulers four deployment radars and two coal plants.

Every scenario remains available through the headless CLI and test suite. The
Bevy/Wasm control deck exposes only the four complete 50x50 world simulations,
so focused component fixtures do not appear as player-facing content. See
[v3-worlds.md](v3-worlds.md) for the world roster and provenance.
