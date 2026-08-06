# Compact game persistence

The compact player loop survives a reload. Without that, the longest signal the
game can produce is a single sitting, which leaves the open design questions
unanswerable: whether the provisional 16x16 map is too small, whether the
building-unlock thresholds pace well, whether market demand grows at the right
rate. Each needs a session that runs long enough to outgrow its start.

## Ownership

`factory_sim` owns the save. It exposes `CompactGame::to_save_string` and
`CompactGame::from_save_string`, so the format is exercised by ordinary crate
tests with no browser involved. `factory_shell::storage` moves that opaque
string to and from the platform and never inspects it. This follows the
simulation-owns-logic rule in [AGENTS.md](../AGENTS.md).

`CompactSnapshot` is not the save format. It is a lossy presentation
projection, so restoring from one would silently drop authoritative state.
The save carries `CompactGame` itself.

## Format

The stored document is a versioned envelope:

```json
{"version":1,"game":{...}}
```

`COMPACT_SAVE_VERSION` starts at 1. Loading reads the version through a probe
struct before parsing the body, so a save written by a future build reports its
version instead of failing with a field-shaped parse error from the current
one. A version mismatch returns `CompactSaveError::UnsupportedVersion` rather
than loading part of a document.

Every field is an ordered collection (`BTreeSet`, `BTreeMap`, `Vec`), so a save
is byte-stable for a given state and round-trip equality is directly testable.

### Item ids

`ItemId` wraps a `&'static str`, which cannot borrow from runtime input, so it
cannot derive `Deserialize`. It carries a resolving implementation that maps the
saved string back to its interned constant through `ItemId::ALL` and rejects
anything unknown. That turns a corrupt or hand-edited save into a clean error
instead of an accepted item that no recipe recognizes. `every_catalog_item_resolves`
guards `ItemId::ALL` against catalog growth.

## Behavior

- The viewer restores the previous session at startup and reports that it did.
- A save this build cannot read is reported in the feedback line, and play starts fresh. The stale slot is left in place so a later build that understands it can still recover the session.
- Autosave runs every five seconds and only when the world actually changed, so a paused session performs no writes. Saving per tick would serialize and store the world up to eight times a second at fast speed.
- Reset clears the stored slot, because reset is the deliberate way to abandon a session.
- A storage failure, including browser quota limits and private-mode rejection, is logged and never interrupts play.

## Storage location

- **Browser** - `localStorage` under the key `factory-game-v3.compact-save`.
- **Native** - `$HOME/.local/share/factory-game-v3/factory-game-v3.compact-save.json`, falling back to the working directory when `HOME` is unset.

## Changing the format

Any change to `CompactGame`'s serialized shape needs `COMPACT_SAVE_VERSION`
bumped in the same commit. Without the bump an in-progress session is read with
the wrong field expectations, which is the exact session persistence exists to
protect.

## See also

- [AGENTS.md](../AGENTS.md)
- [compact-first-playable.md](compact-first-playable.md)
- [FEATURES.md](FEATURES.md)
