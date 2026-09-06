# V3 simulation fixtures

One deterministic 50x50 world remains, as a headless integration fixture. The
player-facing app uses the compact planning scenario described in
[compact-first-playable.md](compact-first-playable.md), so this world does not
appear in the game UI.

## V3 50x50 factory world

The primary world preserves the final C# setup at full scale. It combines 423
seeded ore deposits, a central fifteen-factory district, fifteen haulers, four
radars, and one coal plant. Its broad generated resource field remains the
integration and sustained-operation proof, exercised by `just v2-liveness`.

## The three authored worlds that were here

`legacy-assembly-yard`, `twin-plant-basin` and `four-corners-works` were removed
in `teable:coilyco-gaming/factory-game-v3#7041`. They adapted the earlier and
later C# yards and added a Rust-native distributed layout, and nothing in the
repository ever simulated any of them. The only test that touched them compared
their layout arrays against their own declared counts, which a world cannot fail
by being broken: `four-corners-works` produced zero items in 40 ticks and passed
it anyway.

Migration evidence is what they were for, and git history holds that better than
a fixture nothing runs. Recover them from the history of
`crates/factory_content/src/lib.rs` if a full-scale authored layout is ever
wanted again.

The smaller catalog scenarios remain available to the headless runner and test
suite as focused component fixtures, and
[factory-scenarios.md](factory-scenarios.md) lists them.
