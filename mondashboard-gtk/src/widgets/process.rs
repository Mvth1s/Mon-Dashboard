use gtk4::Box as GtkBox;
use mondashboard_core::process::AllProcessStats;

pub struct ProcessWidget {
    pub container: GtkBox,
}

impl ProcessWidget {
    pub fn new() -> Self {
        todo!()
    }

    pub fn update(&self, _data: &AllProcessStats) {
        todo!()
    }
}
