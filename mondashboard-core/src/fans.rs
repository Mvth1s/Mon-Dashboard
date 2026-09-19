#[derive(Debug, Clone)]
pub enum FanKind {
    CpuCooler,
    CaseFan,
    AioPump,
    AioRadiator,
    GpuFan,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FanStats {
    /// Label from hwmon (e.g. "fan1", "Pump", "CPU Fan").
    pub label: String,
    pub kind: FanKind,
    pub rpm: u32,
    pub rpm_min: Option<u32>,
    pub rpm_max: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct CoolingStats {
    pub fans: Vec<FanStats>,
    /// AIO only, if exposed by the driver.
    pub coolant_temp_celsius: Option<f32>,
}

/// Sources: /sys/class/hwmon/*/fan*_input and fan*_label.
pub fn get_cooling_stats() -> CoolingStats {
    todo!()
}
