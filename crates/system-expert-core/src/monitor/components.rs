//! Hardware temperature sensors (motherboard, CPU package, etc).
//!
//! Sensor availability is platform- and hardware-dependent; `sysinfo` returns
//! an empty list on machines/platforms where no sensors are exposed (this is
//! common on Windows without vendor-specific drivers).

use serde::Serialize;
use sysinfo::Components;

#[derive(Debug, Clone, Serialize)]
pub struct ComponentMetrics {
    pub label: String,
    pub temperature_celsius: Option<f32>,
    pub max_temperature_celsius: Option<f32>,
    pub critical_temperature_celsius: Option<f32>,
}

pub fn collect(components: &Components) -> Vec<ComponentMetrics> {
    components
        .iter()
        .map(|component| {
            let temperature = component.temperature();
            let max = component.max();
            ComponentMetrics {
                label: component.label().to_string(),
                temperature_celsius: (!temperature.is_nan()).then_some(temperature),
                max_temperature_celsius: (!max.is_nan()).then_some(max),
                critical_temperature_celsius: component.critical(),
            }
        })
        .collect()
}
