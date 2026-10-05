# Simulation rules

## World and boundaries

The row-major world has a closed boundary. Out-of-range neighbors do not exist. Cell IDs are zero-based validated indices. Rule order and neighbor order are stable: up, left, right, down. Local updates are asynchronous scheduler quanta, not global ticks. A dormant cell receives no recurring evaluation; movement, ignition, blasts, and accepted edits wake a fixed cardinal neighborhood.

Materials are Air, Stone, Wood, Sand, Explosive, and Water. Stone is fixed until changed by an explicit edit. Burning is a cell-state countdown on wood, not an additional material. Cell reads preserve the existing `state` byte for pending blast energy and add `burning` as an additive state field.

## Granular and water movement

Sand and water inspect the cell directly below. If it is Air, the material moves there atomically. Sand otherwise remains dormant. Water otherwise tries one horizontal neighbor; initial direction is selected by `(x + y) mod 2`, and it tries the opposite direction if blocked. The closed boundary prevents escape. This intentionally simple local spreading rule may oscillate and is not a pressure-fluid model.

A successful move clears the source, fills the destination, and wakes cardinal neighbors of both cells. Sand and water counts are conserved by simulation movement. Explicit paint/reset can change counts. Each evaluation inspects a fixed bounded neighborhood, and no rule searches an unbounded column or region.

## Fire and explosives

Wood ignition sets a 12-service burn countdown. On each serviced burning-wood evaluation, its cardinal neighbors are inspected: adjacent wood is ignited and adjacent explosives receive a blast request with energy 8. The current wood countdown decreases by one; at zero, wood converts to Air. The cell remains wood while burning. Evaluation is rescheduled only while burn time remains. Ignition is idempotent while already burning.

A blast request changes its target to Explosive and merges energy by `max(existing, min(requested, 15))`, not addition. A serviced blast converts the explosive cell to Air. Each cardinal adjacent explosive receives `max(existing, energy - 1)` when energy exceeds one. Adjacent wood is ignited by the blast regardless of remaining propagation energy. Blast energy is finite, saturating, and attenuates by one edge; this is not an additive physical shock wave. Stone, sand, and water are not destroyed by blast in this rules version.

Fire, blast, and edit activity wake only fixed cardinal neighborhoods. All timers advance on service, not wall time. Under overload, different regions progress at different rates; this is not uniform time dilation.

## Coalescing and reset

Duplicate reevaluation requests coalesce to one per-cell pending bit. Blast requests coalesce by maximum energy. Pending state remains authoritative when a ready ring is full and is admitted by the charged recovery cursor. Paint commands to the same cell coalesce to the latest material by a direct per-cell queue-slot index in O(1); a full command ring rejects a new distinct command. The command ring is capped at 256 descriptors. Fixture preparation blocks external mutations until its charged cursor completes or is cancelled.

Reset increments a checked world generation immediately, discards queued command/job frontiers, and blocks new edits while a cursor clears one cell per charged reset quantum. Every old job is generation-tagged and cannot mutate the new generation. Reset progress is observable; cancellation is deliberately not offered for world reset. Starting a fixture schedules reset and then a versioned fixture cursor, both advanced only by `World::step`; fixture progress and cancellation are observable. Reset marks renderer chunks dirty as cells are cleared.

The rules version is `RULE_VERSION = 4`. Determinism is guaranteed only for the same executable/rules, world dimensions, admitted operation order, capacities, policy, and slice schedule. Bounded mode spends at most its allowance. Traditional mode captures the ready frontier at update start, processes that finite frontier without the credit cap, and leaves work created during that update for the next one; it never recursively drains to quiescence.
