use gtk4::Box as GtkBox;
use mondashboard_core::disk::AllDiskStats;

pub struct DiskWidget {
    pub container: GtkBox,
}

impl DiskWidget {
    pub fn new() -> Self {
        todo!()
    }

    pub fn update(&self, _data: &AllDiskStats) {
        todo!()
    }
}
