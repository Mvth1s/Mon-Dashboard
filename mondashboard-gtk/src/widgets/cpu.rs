use gtk4::Box as GtkBox;
use mondashboard_core::cpu::CpuStats;

pub struct CpuWidget {
    pub container: GtkBox,
}

impl CpuWidget {
    pub fn new() -> Self {
        todo!()
    }

    pub fn update(&self, _data: &CpuStats) {
        todo!()
    }
}
