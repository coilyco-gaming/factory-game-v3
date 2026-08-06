# Deployment radar targeting

Deployment radars own target discovery and claims for spawnable mining drills.
Factories remain inventory providers, and haulers remain the typed retrieve and
deploy receivers. This restores the authority split from the retained Unity
design without moving simulation rules into the viewer.

## Authority boundary

Each scenario radar declares a deployment item, a target resource item, and a
grid position. On every deterministic intent refresh, `factory_sim`:

* releases claims whose source is active, exhausted, depleted, missing, or no
  longer matches the radar's target item
* retains valid existing claims before any radar may discover a new target
* visits unclaimed compatible dormant sources by squared distance from the
  radar, with node identity as the stable tie-breaker
* exposes one typed deploy intent for the retained claim
* asks the nearest factory with matching inventory to supply the retrieval

Radars run in scenario order. A shared ordered claim set prevents two radars
from owning one source. A claim persists while inventory is unavailable or a
hauler is in flight, then releases on the tick after deployment activates the
source. Claim and release transitions appear in tick events and the radar's
bounded alert history.

Radar topology nodes are authority markers, not new collision cells. The v2
positions match the historical central radar offsets, including cells already
used by the current abstract road and logistics layout. They remain traversable
so restoring targeting ownership does not invent a new occupancy contract.

## V2 world

The 100x100 scenario starts three mining-drill radars:

* `radar-0` targets iron ore from `(50, 50)`
* `radar-1` targets copper ore from `(50, 51)`
* `radar-2` targets coal from `(50, 52)`

The 50-tick integration proof now continues claiming after the first three
deployments and activates six deposits. Coal-plant targeting remains a
separate expansion slice.

## Projection and scale

Snapshots include every radar's typed configuration, current claim, deploy
intent, and alerts. The Bevy viewer renders radar nodes and their claim labels
through the same projection in detail and whole-map modes.

Target discovery performs one bounded source pass per radar. Topology position
queries use an ordered index while the existing ordered node vector remains the
snapshot source, avoiding repeated full-node scans in the 100x100 tick path.

On the implementation host, five warm release-mode executions of the 50-tick
v2 test reported 0.51 to 0.52 seconds inside the test harness. The pre-change
warm baseline was 1.17 seconds. The later workload is larger because it deploys
six drills instead of three, so this point-in-time comparison demonstrates that
radar ownership did not trade scale behavior for feature coverage. It is not a
portable wall-clock budget.

The focused tests cover nearest-target ownership, competing-radar exclusion,
active, exhausted, and depleted filtering, deterministic replay, claim release,
CLI serialization, and viewer labels. The v2 integration test keeps fixed
material-flow totals across the populated world.
