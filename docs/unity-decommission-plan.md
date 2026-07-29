# Unity decommission plan

The retained C# tree remains a migration reference until a Rust test and viewer
surface cover each player-visible behavior.

## Order

1. The agent expands world mutation to general spawn, move, delete, and
   teardown operations.
2. The agent ports general spawnable output construction.
3. The agent adds player-facing programming and logistics policy controls.
4. The agent proves every retained C# gameplay component is covered or
   explicitly obsolete.
5. The agent removes `Assets/Scripts/`, `tests.csproj`, and C#-only plugins in a
   dedicated deletion change.

Retained textures, materials, and other reusable art have a separate decision.
C# decommission does not require deleting them.

The proof and removed first batch live in
[csharp-decommission.md](csharp-decommission.md).
