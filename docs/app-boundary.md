# App boundary: factory-game vs galaxy generation

`factory-game-v3` is the programming-first factory/logistics simulation. Galaxy
generation is a separate app. The two stay completely separate: separate
repos, separate domain code, separate runtimes. This page records the boundary
Kai set during planning (issue #17) so future issues and specs cite it instead
of re-litigating it.

## What this repo owns

- The deterministic factory simulation kernel (`factory_sim`), its content
  (`factory_content`), and its headless runner (`factory_cli`).
- Any future Bevy surface in this repo is scoped to factory sim state:
  sources, factory, haulers, dispatch protocol, inventory, ticks. Issue #16 is
  a factory-game debug/observability viewer over `factory_sim`, nothing more.
- The Bevy/Wasm app shell (#18) is an ops/build/deploy foundation for this
  app. Similar build/deploy shape to other apps is fine - shared code is not.

## What this repo must not grow

- No `factory_gen` or galaxy-generation crate.
- No galaxy seeds, star systems, galaxy resource distribution, or galaxy
  viewer work. Those belong to a separate repo/app if and when Kai creates
  one.
- No shared runtime coupling with any galaxy app.

## If the apps ever need to exchange data

Prefer exported artifacts over shared code: versioned snapshot/schema files
(the JSONL tick snapshots and run summaries `factory_cli` already emits are
the template). A shared contract crate happens only if Kai explicitly creates
one - neither app reaches into the other's internals.
