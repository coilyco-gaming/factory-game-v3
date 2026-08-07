# Completed C# decommission

The repository removed its Unity gameplay and C# dependency surface after the
Rust workspace passed every parity gate. Git history retains the reference.

## File-to-Rust proof

- `Battery`, `Power`, and `PowerLine` components map to `factory_sim::power`,
  owner batteries, generation, balancing, consumption, and automatic line
  construction tests.
- `Dispatch`, `DispatchReceiver`, `ResourceInserter`, and
  `ResourceRetriever` map to typed intents and receiver phases, nearest-target
  resolution, arbitration, adjacent transfers, and buffer-policy tests.
- `Resources`, `Mining`, and `Production` map to capacity-aware inventories,
  finite or manifest extraction, indexed factories, recipes, and chain tests.
- `Player`, `StatusData`, `StatusUILeft`, and `StatusUIRight` map to the Bevy
  control deck, bounded camera focus, zoom, focused inspection, global totals,
  metrics, and activity feed.
- `SpriteMap`, `GameController`, and `WorldObject` map to typed topology,
  deterministic `GameState` ticks, snapshot projection, and queued spawn,
  movement, deployment, and deletion mutations.
- `FactoryGameContent` and `GameContent` map to the complete typed
  `factory_content` catalog and scenario definitions.
- coal plant, factory, foundry, mining drill, ore, power line, radar, and truck
  classes map respectively to powered nodes, indexed production, sources,
  deployment target resolution, line batteries, and haulers.
- `ExampleComponent` was a C# test fixture. `Util` only humanized strings.
  Unity object wrappers, telemetry bootstrap, and YAML serialization were
  engine plumbing rather than gameplay.

The first movement, pathfinding, and deployment batch had already been removed
after DETOUR and DEPLOY proof. The final batch removed the remaining 64 script
and metadata files, `tests.csproj`, and 1,661 Unity-only plugin and metadata
files. Those plugins supplied C5, PathFinder, xUnit, YAML, .NET extensions, and
OpenTelemetry solely to the retired C# path.

## Preserved assets

The C# deletion did not touch scenes, textures, materials, building and vehicle
art, fonts, shaders, or TextMesh Pro resources. Those assets were kept as inert
migration inputs, to be evaluated separately for Bevy presentation work.

That evaluation happened. The accepted runtime sprites were pulled into
`crates/factory_shell/assets/factory/`, and the rest of the tree was removed in
a later pass, because Bevy cannot read Unity scenes, materials, shader graphs,
ShaderLab shaders, or TextMesh Pro. Git history and the LFS objects on the
canonical remote retain every original image. See
[factory-art.md](factory-art.md).
