# Dispatch Priority Policy

`factory_sim` exposes a viewer-independent control plane for ordering contested
delivery demand. A rule targets one destination and item, so prioritizing iron
ore at one factory does not change iron ore demand anywhere else.

## Configure

Use the presets or any `u8` value:

```rust
game.set_dispatch_priority(
  NodeId::Factory(1),
  IRON_ORE,
  DispatchPriority::HIGH,
);
```

`LOW`, `NORMAL`, and `HIGH` map to 64, 128, and 192. The scheduler accepts the
full 0 through 255 range through `DispatchPriority::new`. Setting `NORMAL` or
calling `clear_dispatch_priority` removes the sparse override.

## Arbitration

Every tick, active delivery demand is ordered by:

1. descending priority
2. destination node
3. item ID

The scheduler then subtracts stocked and in-flight cargo before assigning
available haulers in ID order. Equal inputs therefore produce equal assignments
without double-serving demand.

## Observe

Resolved numeric priority is serialized on each active `DispatchIntent` and
`DispatchAssignment`. Tick snapshots and CLI JSON preserve the decision input
alongside the chosen source, destination, item, and phase. Existing scenarios
use `NORMAL` until a caller installs an override.

## See also

- [factory-sim.md](factory-sim.md)
- [FEATURES.md](FEATURES.md)
