# Factory simulation scenarios

The deterministic showcase covers eight layouts:

- **Iron bars** - one hauler supplies one foundry from a finite iron deposit.
- **Iron bars fleet** - three haulers arbitrate the same bounded demand.
- **Building materials** - two source types satisfy a multi-input recipe.
- **Powered ironworks** - coal logistics charge an energy-gated factory grid.
- **Drill deployment** - a hauler retrieves a spawnable drill, deploys it at a
  dormant source, and drains the source before teardown.
- **Obstacle convoy** - two haulers route around an occupied cell and arbitrate
  a shared transit cell.
- **Drill production chain** - adjacent inserters convert iron and copper ore
  through five indexed factories into mining drills.
- **Distributed frame line** - separated factories force haulers to discover
  foundry output, retrieve it from an adjacent cell, cross the grid, and satisfy
  downstream frame demand.

Every scenario is available through the headless CLI and the Bevy/Wasm control
deck. The viewer cycles through them automatically after quiet completion.
