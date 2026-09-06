# Stranded products

`liveness_summary().stranded_products` names every item a factory is jammed
holding that nothing in that world can ever ask for. It is a structural
condition rather than a transient one, so once it appears it stays.

An item is stranded when all three hold:

- a factory is blocked with `ProductionBlockReason::OutputFull`,
- no factory in the world lists that item among its recipe inputs, and
- the item is not `can_spawn_game_object`, so it cannot leave by placement.

## Why the field exists

The dispatcher is demand-pull. A factory publishes a collect intent for its
output as an **offer of supply**, and `assign_dispatch` only ever reads that
offer while serving some other factory's `Deliver` demand for the same item. A
world whose deepest recipe produces something nothing downstream consumes has
no such demand, the offer stands unread, and the factory holds its output
buffer forever.

Nothing was wrong with that until you tried to read a run. A stalled world and
a finished world both present as rising `idle_ticks` and a repeating
`product output full` alert, and neither says which one you are looking at.
Seven scenarios were sitting in this state and it had never been noticed,
because the runner defaults to `--ticks 6` and the stall begins at tick 7.

## What it does not mean

A stranded product is not a dispatcher defect and not something the sim can
repair from inside. The two ways out both live in content:

- give the world a downstream recipe that consumes the item, or
- terminate the chain in a spawnable building, which exits by placement.

The first only helps when the new consumer is itself consumed or spawnable.
`distributed-chain` adds a frames factory downstream of the same foundry and
still reaches 339 idle ticks in 400, because `frames` is then the stranded
product instead of `iron_bars`. Moving a stall is not clearing one.

## Reading it

```
just cargo-run run --scenario iron-bars --ticks 40
```

The last line of a run carries `summary` and `liveness`. A healthy chain
reports `"stranded_products": []`. `iron-bars` reports
`"stranded_products": ["iron_bars"]` from tick 7 onward, alongside the 33 idle
ticks that were the only previous evidence.

## See also

- [docs/factory-scenarios.md](factory-scenarios.md)
- [docs/dispatch-policy.md](dispatch-policy.md)
- [docs/headless-runner.md](headless-runner.md)
