fn main() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }))
    .expect("request adapter");
    let info = adapter.get_info();
    let features = adapter.features();
    println!(
        "backend={:?} name={} device_type={:?}",
        info.backend, info.name, info.device_type
    );
    println!("features={features:?}");
    println!(
        "timestamp_query={}",
        features.contains(wgpu::Features::TIMESTAMP_QUERY)
    );
    println!(
        "timestamp_query_inside_encoders={}",
        features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
    );
    println!(
        "timestamp_query_inside_passes={}",
        features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES)
    );
}
