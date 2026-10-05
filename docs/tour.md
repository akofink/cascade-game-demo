# Interactive feature tour

These small worlds run the repository's actual `cascade-sim` compiled to WebAssembly. They are deliberately zoomed in: colors show material, amber highlights show work waiting to be evaluated, orange cells show queued blast energy, and brightened cells show work executed in the last slice (up to 128 sampled cells). Each canvas is also shown as a checked-in image first, so the explanation remains useful before the browser module starts.

<figure>
  <img src="tour/images/cells.png" alt="A small world with sand, water, wood, and a blast">
  <figcaption>Cells and local rules: the screenshot shows the starting materials before movement and burning settle.</figcaption>
</figure>

**Palette:** Air `#17212b`, stone `#78838d`, wood `#8d4e31`, sand `#e5b84f`, explosive `#f05832`, water `#45a9c5`. Amber highlights mark pending evaluation; hot orange shows pending blast energy or burning heat; brightened cells mark the latest slice's execution sample.

## Cells and local rules

Sand and water inspect only their local neighborhood when selected for evaluation. Fire is a timer on wood; a blast carries bounded, attenuating energy. Dormant cells do no recurring work, while a mutation wakes nearby cells for reevaluation. This is intentionally asynchronous local simulation, not a globally synchronized physics tick. Click a cell to ignite it; shift-click to paint the selected material.

<div class="tour-widget" data-scene="0" data-label="Cells and local rules"></div>

<figure>
  <img src="tour/images/credits.png" alt="A budgeted world with pending cells highlighted">
  <figcaption>Credit budget: pending evaluations wait when the remaining allowance cannot pay for their complete quantum.</figcaption>
</figure>

## The credit budget

Every bounded slice charges scheduler probes and each whole quantum before it runs. Work may stop before the allowance is exhausted if no ready quantum fits; it never spends more than the allowance. Lower the slider to make the difference visible. Step advances exactly one slice, without relying on a frame timer.

<div class="tour-widget" data-scene="1" data-label="The credit budget"></div>

<figure>
  <img src="tour/images/deferral.png" alt="A deferred work ring around a pending blast">
  <figcaption>Deferral: cells beyond the ready ring remain represented by pending state for incremental recovery.</figcaption>
</figure>

## Deferral and recovery

The ready rings have fixed capacity (64 slots per lane in this small tour fixture, intentionally smaller than the native profile). When a ring fills, each affected cell retains its pending bit and a recovery cursor incrementally finds work to admit as slots open. The oldest-pending age reports slices since the oldest outstanding channel was first marked. It is not a prediction of completion time; sustained demand can keep backlog growing.

<div class="tour-widget" data-scene="2" data-label="Deferral and recovery"></div>

<figure>
  <img src="tour/images/comparison.png" alt="Bounded and traditional simulation backlogs compared">
  <figcaption>Same small trigger, two scheduler policies. Browser step durations vary with the machine and are illustrative only.</figcaption>
</figure>

## Bounded versus traditional

Both worlds use the same real rules and the same initial trigger. Bounded mode obeys its credit allowance. Traditional mode captures and executes its full pending frontier at slice entry, including a full-world scan, and may take longer as the world grows. Compare credits, executed work, measured browser slice time, and pending backlog history below. Browser timing is illustrative, not the native benchmark: use the [performance report](performance.md) for measured benchmark methodology and results. A run completes when its pending work reaches zero.

<div class="tour-comparison" data-scene="3" data-label="Paired policy comparison"></div>

<figure>
  <img src="tour/images/destroy-performance.png" alt="The miniature overload scene under repeated blasts">
  <figcaption>DESTROY PERFORMANCE in a 128 by 128 world: repeated disturbances build visible backlog without allocating a new world.</figcaption>
</figure>

## DESTROY PERFORMANCE, in miniature

The native app provides the full-scale counterpart:

<figure>
  <img src="cascade-overload.png" alt="Native Cascade app during a traditional overload comparison">
  <figcaption>Native app screenshot from the existing overload smoke path; unlike the browser illustrations, the benchmark evidence is reported separately.</figcaption>
</figure>

The button admits a capped stream of repeated cell disturbances into the same fixed world. Releasing it stops new requests; accepted work remains pending. This miniature illustrates overload and recovery without claiming full-size timing or resource results.

<div class="tour-widget" data-scene="4" data-label="DESTROY PERFORMANCE"></div>

<figure>
  <img src="tour/images/player-focus.png" alt="A player action surrounded by cells awaiting evaluation">
  <figcaption>Player focus is a placeholder until the simulation's focus API is merged.</figcaption>
</figure>

## Player focus

The simulation's player-focus API is still in development. Until that API lands on `main`, this page intentionally makes no claim about preferential scheduling or focus regions.

<div class="tour-placeholder" role="note">Player focus visualization will appear here after the focus API is available.</div>

<script type="module" src="tour/tour.js"></script>
<link rel="stylesheet" href="tour/tour.css">
