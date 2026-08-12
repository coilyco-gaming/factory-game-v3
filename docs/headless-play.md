# Headless play surface

`factory_cli play` drives the compact first-playable loop over stdin and
stdout, so the game a person plays in the viewer can also be played by a
script. It is the runner reach the compact loop previously lacked: the only
client of `CompactGame` was the Bevy shell, so no determinism proof covered
the loop that ships as the headline feature.

```bash
ward exec play
```

The session reads one JSON request per line and writes exactly one response
per line, so a driver never has to guess how many lines an action produced.

## Requests

* `observe` - no fields - return the world without advancing it.
* `step` - `ticks`, default 1 - advance the simulation.
* `place_road` - `x`, `y` - lay a road cell.
* `remove_road` - `x`, `y` - lift a road cell.
* `place_building` - `x`, `y` - place a factory and return its id.
* `configure_building` - `building`, `recipe` - set `IronBars` or `CopperBars`.
* `save` - no fields - return the versioned save string.
* `quit` - no fields - end the session.

## Responses

Requests dispatch against `factory_sim`, which keeps every rule. Every
response carries `ok` and a full `snapshot`. A successful edit may add
`changed` or `building`, and `save` adds the save string.

A refused edit is an ordinary turn result rather than a transport failure. The
process stays up, `ok` is `false`, and the line carries a stable `error_kind`
next to the human-readable `error`:

```json
{"ok":false,"error_kind":"road_required","error":"building at 0,0 needs road frontage","snapshot":{...}}
```

The kinds are `out_of_bounds`, `cell_occupied`, `road_in_use`,
`road_required`, `building_allowance_exhausted`, `unknown_building`,
`tick_budget_exhausted`, and `malformed_request`. Input that does not parse is
answered the same way, so a driver emitting bad JSON gets a correction rather
than a dead pipe. Events drain on every snapshot, so a multi-tick `step`
re-attaches the events its intermediate ticks would otherwise have discarded.

## Playing well

Deposits mine themselves into a stockpile. Trucks do the rest, and they only
traverse road cells, so earning anything requires a road route from the
starting stub to a deposit and on to a factory with road frontage. A road that
reaches no deposit produces no revenue no matter how long it runs. The starter
world is 16x16, warehouse at 8,8, iron deposits at 2,2 and 2,13. A column up
x=7 and a row west along y=2 reaches the first one:

```json
{"action":"place_road","x":7,"y":8}
{"action":"place_building","x":6,"y":3}
{"action":"configure_building","building":0,"recipe":"IronBars"}
{"action":"step","ticks":600}
```

That route banks 5890 revenue over 600 ticks and lifts the building allowance
from 2 to 6 through the sales unlock ladder. `tests/play.rs` asserts those
exact figures, which makes the loop a determinism proof, not a smoke test.

## Session bounds and resuming

`--max-ticks` caps ticks spent in one session and defaults to 2000. It counts
this session's ticks, not the loaded world's age, so a resumed game gets a
fresh allotment. A `step` past the remaining budget is refused with
`tick_budget_exhausted` and changes nothing.

`--load <file>` starts from a save string and `--save <file>` writes one at
exit, so a session can be branched, replayed, or continued. The format is the
versioned one in [compact-persistence.md](compact-persistence.md).

See [compact-first-playable.md](compact-first-playable.md) for the player-loop
contract and [headless-runner.md](headless-runner.md) for the scenario runner.
