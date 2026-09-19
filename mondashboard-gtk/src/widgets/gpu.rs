use gtk4::Box as GtkBox;
use mondashboard_core::gpu::AllGpuStats;

pub struct GpuWidget {
    pub container: GtkBox,
}

impl GpuWidget {
    pub fn new() -> Self {
        todo!()
    }

    pub fn update(&self, _data: &AllGpuStats) {
        todo!()
    }
}
