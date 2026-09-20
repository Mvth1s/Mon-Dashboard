use serde::{Deserialize, Serialize};

use crate::alerts::AlertConfig;

/// D'où viennent les informations de batterie. Se change dans le fichier de
/// configuration, sans recompiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BatterySource {
    /// Essaie UPower, retombe sur sysfs si le service est absent.
    #[default]
    Auto,
    /// Lecture directe de /sys/class/power_supply.
    Sysfs,
    /// Service UPower via D-Bus.
    UPower,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TemperatureUnit {
    #[default]
    Celsius,
    Fahrenheit,
}

/// Géométrie de la fenêtre, retenue d'une session à l'autre. Les
/// gestionnaires de fenêtres ne la restaurent pas tous, et ceux qui le font
/// ne s'accordent pas : l'application s'en charge elle-même.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: i32,
    pub height: i32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1180,
            height: 800,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverlayConfig {
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_vram: bool,
    pub show_ram: bool,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetSpecificConfig {
    Cpu {
        show_per_core: bool,
    },
    Gpu {
        selected_gpu_index: Option<usize>,
    },
    Network {
        interface: Option<String>,
        ping_host: Option<String>,
    },
    Process {
        sort_by_cpu: bool,
        max_rows: u32,
    },
    Fans {
        hidden_labels: Vec<String>,
    },
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPlacement {
    /// e.g. "cpu", "gpu_0", "disk_sda"
    pub widget_id: String,
    pub col: u32,
    pub row: u32,
    pub col_span: u32,
    pub row_span: u32,
    pub visible: bool,
    pub widget_config: WidgetSpecificConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub name: String,
    pub placements: Vec<WidgetPlacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Bumped on breaking schema changes for migration.
    pub version: u32,
    pub refresh_interval_secs: u32,
    pub temperature_unit: TemperatureUnit,
    pub ping_host: String,
    #[serde(default)]
    pub battery_source: BatterySource,
    #[serde(default)]
    pub window: WindowConfig,
    pub alerts: AlertConfig,
    pub layouts: Vec<LayoutConfig>,
    pub active_layout: String,
    pub overlay: OverlayConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            refresh_interval_secs: 2,
            temperature_unit: TemperatureUnit::Celsius,
            ping_host: "1.1.1.1".to_string(),
            battery_source: BatterySource::default(),
            window: WindowConfig::default(),
            alerts: AlertConfig::default(),
            layouts: vec![],
            active_layout: "Default".to_string(),
            overlay: OverlayConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_par_defaut_serialisable() {
        let json = serde_json::to_string(&AppConfig::default()).expect("sérialisation");
        let relu: AppConfig = serde_json::from_str(&json).expect("désérialisation");
        assert_eq!(relu.refresh_interval_secs, 2);
        assert_eq!(relu.battery_source, BatterySource::Auto);
    }

    #[test]
    fn source_batterie_absente_retombe_sur_auto() {
        // Une config écrite par une version antérieure reste lisible.
        let json = r#"{"version":1,"refresh_interval_secs":2,"temperature_unit":"Celsius",
            "ping_host":"1.1.1.1","alerts":{"cpu_threshold_percent":null,"cpu_duration_secs":0,
            "ram_threshold_percent":null,"gpu_temp_threshold_celsius":null,
            "gpu_usage_threshold_percent":null,"disk_free_threshold_gb":null,
            "ping_threshold_ms":null,"battery_threshold_percent":null,
            "fan_stopped_threshold_rpm":null,"cooldown_secs":0},"layouts":[],
            "active_layout":"Default","overlay":{"show_cpu":false,"show_gpu":false,
            "show_vram":false,"show_ram":false,"opacity":0.0}}"#;
        let config: AppConfig = serde_json::from_str(json).expect("désérialisation");
        assert_eq!(config.battery_source, BatterySource::Auto);
    }
}
