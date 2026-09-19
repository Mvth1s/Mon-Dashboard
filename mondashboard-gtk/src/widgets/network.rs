use gtk4::Box as GtkBox;
use mondashboard_core::network::NetworkStats;

pub struct NetworkWidget {
    pub container: GtkBox,
}

impl NetworkWidget {
    pub fn new() -> Self {
        todo!()
    }

    pub fn update(&self, _data: &NetworkStats) {
        todo!()
    }
}
