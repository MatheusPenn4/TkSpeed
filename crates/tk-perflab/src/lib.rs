//! TkPerformanceLab — fundação de medição de performance.
//!
//! PL-1 (CPU) + PL-2 (RAM/Storage/Completo + GPU via NVML + temperaturas +
//! detectores). SEM FPS/ETW/elevação/otimização.
//!
//! Princípio: nenhum ganho é afirmado sem evidência reproduzível e significativa.

pub mod aggregate;
pub mod benchmark;
pub mod collector;
pub mod compare;
pub mod confidence;
pub mod detect;
pub mod frame;
pub mod hardware;

pub use aggregate::{aggregate, Aggregate};
pub use benchmark::{run_benchmark, run_complete, run_cpu, run_io, run_ram, SUITE_VERSION};
pub use collector::{MetricPoint, MetricsCollector, SysinfoCollector};
pub use compare::compare;
pub use confidence::{build_noise_profile, noise_cv};
pub use detect::detect_bottleneck;
pub use frame::{
    demo_trace, frame_stats, run_fps_capture, FrameSource, PresentMonFrameSource, ReplayFrameSource,
    FPS_SUITE,
};
pub use hardware::{hardware_snapshot, GpuCollector};
