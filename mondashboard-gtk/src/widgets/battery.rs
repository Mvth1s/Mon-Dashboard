use gtk4::Box as GtkBox;
use mondashboard_core::battery::BatteryStats;

/// Hidden automatically when BatteryStats::present is false (desktop).
pub struct BatteryWidget {
    pub container: GtkBox,
}

impl BatteryWidget {
    pub fn new() -> Self {
        todo!()
    }

    pub fn update(&self, _data: &BatteryStats) {
        todo!()
    }
}
