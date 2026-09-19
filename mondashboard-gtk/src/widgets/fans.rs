use gtk4::Box as GtkBox;
use mondashboard_core::fans::CoolingStats;

/// Hidden automatically when no fan sensors are detected.
pub struct FansWidget {
    pub container: GtkBox,
}

impl FansWidget {
    pub fn new() -> Self {
        todo!()
    }

    pub fn update(&self, _data: &CoolingStats) {
        todo!()
    }
}
