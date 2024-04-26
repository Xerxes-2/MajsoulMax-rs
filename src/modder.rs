use once_cell::sync::Lazy;

use crate::settings::ModSettings;

pub static MOD_SETTINGS: Lazy<ModSettings> = Lazy::new(ModSettings::new);

pub struct Modder {
    
}
