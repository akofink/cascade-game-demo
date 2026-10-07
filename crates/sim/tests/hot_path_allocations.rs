use cascade_sim::{Capacity, Command, Credits, Material, SchedulerPolicy, SubmitResult, World};
use stats_alloc::{Region, StatsAlloc};
use std::alloc::System;

#[global_allocator]
static ALLOCATOR: StatsAlloc<System> = StatsAlloc::system();

#[test]
fn saturated_simulation_and_scheduler_slices_do_not_grow_heap_after_initialization() {
    let mut world = World::new_with_policy(
        256,
        256,
        Credits::new(25),
        Capacity::new(32_768),
        Capacity::new(256),
        SchedulerPolicy::Bounded,
    )
    .expect("world initialization");

    for index in 0..256u32 {
        let index = index * 97 % 65_536;
        let cell = world
            .cell_id(index % 256, index / 256)
            .expect("in-bounds command target");
        assert_eq!(
            world.submit(Command::Paint {
                cell,
                material: Material::Explosive,
            }),
            SubmitResult::Accepted
        );
    }
    for _ in 0..16 {
        world.step();
    }

    let region = Region::new(&ALLOCATOR);
    let mut observed_saturated_slice = false;
    for _ in 0..256 {
        let metrics = world.step();
        assert!(metrics.charged <= metrics.allowed);
        observed_saturated_slice |= metrics.evaluations + metrics.blasts > 0;
    }
    let allocations = region.change();
    assert!(
        observed_saturated_slice,
        "test did not execute simulation work"
    );
    assert_eq!(
        allocations.allocations, 0,
        "unexpected heap allocation: {allocations:?}"
    );
    assert_eq!(
        allocations.reallocations, 0,
        "unexpected heap growth: {allocations:?}"
    );
    assert_eq!(
        allocations.bytes_allocated, 0,
        "unexpected allocated bytes: {allocations:?}"
    );
}
