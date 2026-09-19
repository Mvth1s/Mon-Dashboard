use gtk4::Box as GtkBox;
use mondashboard_core::memory::MemoryStats;

pub struct MemoryWidget {
    pub container: GtkBox,
}

impl MemoryWidget {
    pub fn new() -> Self {
        todo!()
    }

    pub fn update(&self, _data: &MemoryStats) {
        todo!()
    }
}
