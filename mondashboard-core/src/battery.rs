#[derive(Debug, Clone)]
pub enum BatteryState {
    Charging,
    Discharging,
    Full,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct BatteryStats {
    /// false on desktops — widget is hidden when false.
    pub present: bool,
    pub percentage: f32,
    pub state: BatteryState,
    pub time_to_empty_min: Option<u32>,
    pub time_to_full_min: Option<u32>,
    pub energy_wh: f64,
    pub energy_full_wh: f64,
    pub energy_full_design_wh: f64,
    /// energy_full / energy_full_design * 100
    pub health_percent: f32,
    pub cycle_count: Option<u32>,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub technology: Option<String>,
}

impl Default for BatteryStats {
    fn default() -> Self {
        Self {
            present: false,
            percentage: 0.0,
            state: BatteryState::Unknown,
            time_to_empty_min: None,
            time_to_full_min: None,
            energy_wh: 0.0,
            energy_full_wh: 0.0,
            energy_full_design_wh: 0.0,
            health_percent: 0.0,
            cycle_count: None,
            vendor: None,
            model: None,
            technology: None,
        }
    }
}

/// Data sourced from UPower via D-Bus (org.freedesktop.UPower).
pub fn get_battery_stats() -> BatteryStats {
    todo!()
}
