# Deployment radar targeting

Deployment radars own target discovery and claims for mining drills and remote
coal plants. Factories remain inventory providers, and haulers remain the typed
retrieve and deploy receivers. This restores the retained authority split
without moving simulation rules into the viewer.

## Authority boundary

Each scenario radar declares a deployment item, a target resource item, and a
grid position. On every deterministic intent refresh, `factory_sim`:

* releases claims whose source is active, occupied, exhausted, depleted,
  missing, or no longer matches the radar's target item
* retains valid existing claims before any radar may discover a new target
* visits unclaimed compatible dormant sources by squared distance from the
  radar, with node identity as the stable tie-breaker
* exposes one typed deploy intent for the retained claim
* asks the nearest factory with matching inventory to supply the retrieval

Radars run in scenario order. A shared ordered claim set prevents two radars
from owning one source. A claim persists while inventory is unavailable or a
hauler is in flight, then releases on the tick after deployment activates or
occupies the source. Claim and release transitions appear in tick events and
the radar's bounded alert history.

Radar topology nodes are authority markers, not new collision cells. The v2
positions match the historical central radar offsets, including cells already
used by the current abstract road and logistics layout. They remain traversable
so restoring targeting ownership does not invent a new occupancy contract.

## V2 world

The 100x100 scenario starts four deployment radars:

* `radar-0` targets iron ore from `(50, 50)`
* `radar-1` targets copper ore from `(50, 51)`
* `radar-2` targets coal with mining drills from `(50, 52)`
* `radar-3` targets coal with coal plants from `(50, 53)`

The 50-tick integration proof continues claiming after the first three drill
deployments and activates six deposits. The longer construction lifecycle is
covered in [remote-coal-plants.md](remote-coal-plants.md).

## Projection and scale

Snapshots include every radar's typed configuration, current claim, deploy
intent, and alerts. The Bevy viewer renders radar nodes and their claim labels
through the same projection in detail and whole-map modes.

Target discovery performs one bounded source pass per radar. Topology position
queries use an ordered index while the existing ordered node vector remains the
snapshot source, avoiding repeated full-node scans in the 100x100 tick path.

On the implementation host, the 50-tick release test now finishes in 0.15
seconds inside the test harness. The earlier post-radar baseline was 0.51 to
0.52 seconds. Deterministic route caching and sealed-endpoint filtering account
for the later reduction. This is a point-in-time comparison rather than a
portable wall-clock budget.

The focused tests cover nearest-target ownership, competing-radar exclusion,
active, exhausted, and depleted filtering, deterministic replay, claim release,
CLI serialization, and viewer labels. The v2 integration test keeps fixed
material-flow totals across the populated world.
