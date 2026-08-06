# V3 simulation fixtures

The four deterministic 50x50 worlds remain headless integration fixtures. The
player-facing app now uses the compact planning scenario described in
[compact-first-playable.md](compact-first-playable.md). These worlds preserve
large-scale migration evidence without appearing in the game UI.

## V3 50x50 factory world

The primary world preserves the final C# setup at full scale. It combines 423
seeded ore deposits, a central fifteen-factory district, fifteen haulers, four
radars, and one coal plant. Its broad generated resource field remains the
integration and sustained-operation proof.

## Legacy assembly yard

This authored layout adapts the earlier C# row yard. Three active ore fields
feed a compact three-row assembly block with a single coal plant and eight
haulers. It provides the clearest immediately active production view.

## Twin plant basin

This authored layout adapts the later C# twin-generator yard. Three radars and
twelve haulers deploy drills into clustered iron, copper, and coal basins. Two
coal plants anchor the compact production district.

## Four corners works

This Rust-native authored layout splits iron across the western corners and
copper across the eastern corners. Coal lines the north and south edges. The
central works distributes six foundries, six downstream factories, two coal
plants, four radars, and sixteen haulers across a wider footprint.

The smaller catalog scenarios also remain available to the headless runner and
test suite as focused component fixtures.
