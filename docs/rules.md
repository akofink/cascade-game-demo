# Milestone 1 rules

## World and materials

The row-major world has a closed boundary. Materials are Air, Stone, Wood, Sand, and Explosive. Stone and Wood are inert in this milestone. Cell IDs are zero-based validated indices. The compact cell record contains a material byte. The read-only cell view reports pending blast energy as its state byte; energy is held in fixed per-cell pending storage until its job is serviced.

## Sand

A sand evaluation examines only the cell directly below. If it is inside the world and Air, the two materials swap atomically. At the bottom boundary or above an occupied cell, sand remains in place. A move wakes only the four cardinal neighbors of the source and destination. Thus each successful move conserves sand and performs fixed fan-out work.

## Explosives

A blast request sets the target to Explosive and merges its energy by `max(existing, min(requested, 15))`, not addition. A serviced blast turns its own cell to Air. If energy is greater than one, each cardinal in-bounds Explosive neighbor receives `max(existing, energy - 1)`. Energy therefore attenuates by one per serviced edge and cannot propagate beyond its finite starting energy. Other materials are not destroyed by the blast in milestone 1. Blast jobs wake cardinal neighbors after processing.

Rule order, cardinal-neighbor order, queue insertion order, and tie-breaking are fixed. Duplicate local reevaluations coalesce to one pending flag. Blast energy coalesces by maximum. These semantics model bounded local work, not a physically additive shock wave or globally synchronized tick.

## Commands and overload

Paint is a single-cell command. Pending paints for the same cell coalesce to the most recently submitted material; accepted commands are not evicted. A full queue rejects a new distinct paint and increments the rejection counter. Paint changes material when serviced, then requests reevaluation and wakes cardinal neighbors. If the ready ring is full, pending state remains authoritative until the recovery cursor reaches that cell and admits it.
