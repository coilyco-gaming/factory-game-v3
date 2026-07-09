# Factory Game V2

This repository is in transition from Unity/C# to Rust/Bevy. Unity project settings and editor/package scaffolding have been removed; gameplay source and reusable assets remain as migration references.

## Tests

```bash
dotnet watch test tests.csproj --project tests.csproj -- -v quiet --nologo -l:"console;verbosity=normal"
```
