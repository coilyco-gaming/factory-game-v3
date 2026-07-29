# Unity decommission result

The Rust migration completed every decommission gate.

## Completed order

1. Rust tests and the viewer proved each retained gameplay behavior.
2. The audit marked Unity engine plumbing and never-shipped editing controls
   obsolete.
3. The repository removed `Assets/Scripts/`, `tests.csproj`, and all C#-only
   plugins.
4. The repository retained reusable visual assets for a separate art decision.

The full proof and deletion inventory live in
[csharp-decommission.md](csharp-decommission.md).
