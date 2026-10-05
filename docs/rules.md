# Simulation rules

## World and boundaries

The row-major world has a closed boundary. Out-of-range neighbors do not exist. Cell IDs are zero-based validated indices. Rule order and neighbor order are stable: up, left, right, down. Local updates are asynchronous scheduler quanta, not global ticks. A dormant cell receives no recurring evaluation; movement, ignition, blasts, and accepted edits wake a fixed cardinal neighborhood.

Materials are Air, Stone, Wood, Sand, Explosive, and Water. Stone is fixed until changed by an explicit edit. Burning is a cell-state countdown on wood, not an additional material. Cell reads preserve the existing `state` byte for pending blast energy and add `burning` as an additive state field.

## Granular and water movement

Sand and water inspect the cell directly below. If it is Air, the material moves there atomically. Sand otherwise remains dormant. Water otherwise tries one horizontal neighbor; initial direction is selected by `(x + y) mod 2`, and it tries the opposite direction if blocked. The closed boundary prevents escape. This intentionally simple local spreading rule may oscillate and is not a pressure-fluid model.

A successful move clears the source, fills the destination, and wakes cardinal neighbors of both cells. Sand and water counts are conserved by simulation movement. Explicit paint/reset can change counts. Each evaluation inspects a fixed bounded neighborhood, and no rule searches an unbounded column or region.

<figure>
  <img src="rules-images/sand-gravity.png" alt="Sand before and after twelve local simulation slices">
  <figcaption>Left: seeded sand over a stone floor. Right: after 12 slices, serviced grains have fallen through local gravity.</figcaption>
</figure>

<figure>
  <img src="rules-images/water-spread.png" alt="Water before and after twelve local simulation slices">
  <figcaption>Left: a narrow water column. Right: after 12 slices, water has fallen and spread to locally available cells.</figcaption>
</figure>

## Fire and explosives

Wood ignition sets a 12-service burn countdown. On each serviced burning-wood evaluation, its cardinal neighbors are inspected: adjacent wood is ignited and adjacent explosives receive a blast request with energy 8. The current wood countdown decreases by one; at zero, wood converts to Air. The cell remains wood while burning. Evaluation is rescheduled only while burn time remains. Ignition is idempotent while already burning.

A blast request changes its target to Explosive and merges energy by `max(existing, min(requested, 15))`, not addition. A serviced blast converts the explosive cell to Air. Each cardinal adjacent explosive receives `max(existing, energy - 1)` when energy exceeds one. Adjacent wood is ignited by the blast regardless of remaining propagation energy. Blast energy is finite, saturating, and attenuates by one edge; this is not an additive physical shock wave. Stone, sand, and water are not destroyed by blast in this rules version.

<figure>
  <img src="rules-images/wood-fire.png" alt="Wood before ignition and after five simulation slices">
  <figcaption>Left: a wood row. Right: after ignition and 5 slices, burning wood is still wood while its service countdown advances and neighbors wake.</figcaption>
</figure>

<figure>
  <img src="rules-images/explosive-ignition.png" alt="Explosives before and after one ignition slice">
  <figcaption>Left: an explosive row. Right: after ignition and 1 slice, the first explosive has been serviced and propagation is pending.</figcaption>
</figure>

<figure>
  <img src="rules-images/blast-attenuation.png" alt="Blast energy before and after four propagation slices">
  <figcaption>Left: a triggered blast with pending energy. Right: after 4 slices, propagation energy has attenuated along the explosive chain.</figcaption>
</figure>

Fire, blast, and edit activity wake only fixed cardinal neighborhoods. All timers advance on service, not wall time. Under overload, different regions progress at different rates; this is not uniform time dilation.

## Coalescing and reset

Duplicate reevaluation requests coalesce to one per-cell pending bit. Blast requests coalesce by maximum energy. Pending state remains authoritative when a ready ring is full and is admitted by the charged recovery cursor. Paint commands to the same cell coalesce to the latest material by a direct per-cell queue-slot index in O(1); a full command ring rejects a new distinct command. The command ring is capped at 256 descriptors. Fixture preparation blocks external mutations until its charged cursor completes or is cancelled.

Reset increments a checked world generation immediately, discards queued command/job frontiers, and blocks new edits while a cursor clears one cell per charged reset quantum. Every old job is generation-tagged and cannot mutate the new generation. Reset progress is observable; cancellation is deliberately not offered for world reset. Starting a fixture schedules reset and then a versioned fixture cursor, both advanced only by `World::step`; fixture progress and cancellation are observable. Reset marks renderer chunks dirty as cells are cleared.

<figure>
  <img src="rules-images/dormancy-waking.png" alt="Dormant sand and a nearby cell before and after a wake request">
  <figcaption>Left: settled sand is dormant. Right: after 1 slice, a local edit has woken its neighborhood for bounded reevaluation.</figcaption>
</figure>

<figure>
  <img src="rules-images/reset-preparation.png" alt="A prepared wood region before and partway through incremental reset">
  <figcaption>Left: a seeded wood region. Right: after 4 reset slices, the clearing cursor has removed only part of the world.</figcaption>
</figure>

<figure>
  <img src="rules-images/fixture-preparation.png" alt="A wood scene before and during explosive-lattice fixture preparation">
  <figcaption>Left: a prepared wood region. Right: after 6 slices of incremental reset and fixture preparation, part of the seeded explosive lattice is ready.</figcaption>
</figure>

Regenerate these deterministic, 32 by 24, simulation-derived illustrations with `cargo run --locked -p cascade-rule-shots -- docs/rules-images`. Each strip compares the captured setup state with the state after the fixed number of `World::step` calls in `crates/rule-shots/src/main.rs`; colors and overlays match the interactive tour.
The rules version is `RULE_VERSION = 4`. Determinism is guaranteed only for the same executable/rules, world dimensions, admitted operation order, capacities, policy, and slice schedule. Bounded mode spends at most its allowance. Traditional mode snapshots the per-cell pending evaluation and blast channels plus the command count at update start, processes that entire captured frontier without the credit cap, and leaves work created during the update for the next one. It charges and times both full-world scans and never recursively drains to quiescence.
