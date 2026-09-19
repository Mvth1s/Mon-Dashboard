use sysinfo::System;

#[derive(Debug, Clone)]
pub struct CpuStats {
    pub model: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub global_usage: f32,
    pub per_core_usage: Vec<f32>,
    pub frequency_mhz: u64,
    pub frequency_max_mhz: u64,
    pub temperature_celsius: Option<f32>,
}

pub fn get_cpu_stats(sys: &System) -> CpuStats {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_stats_fields_are_present() {
        let stats = CpuStats {
            model: String::new(),
            physical_cores: 0,
            logical_cores: 0,
            global_usage: 0.0,
            per_core_usage: vec![],
            frequency_mhz: 0,
            frequency_max_mhz: 0,
            temperature_celsius: None,
        };
        assert_eq!(stats.global_usage, 0.0);
    }
}
