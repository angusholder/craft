use std::cell::RefCell;
use std::ops::Deref;

use imgui::Ui;


pub struct ImGuiDeref;

impl Deref for ImGuiDeref {
    type Target = Ui<'static>;

    fn deref(&self) -> &Self::Target {
        unsafe {
            Ui::current_ui().unwrap()
        }
    }
}

#[allow(non_upper_case_globals)]
pub static ui: ImGuiDeref = ImGuiDeref;


#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub mouse_sensitivity: f32,
    pub chunk_render_distance: i32,
    pub move_speed: f32,
    pub near: f32,
    pub far: f32,
}

pub static mut SETTINGS_MUT: Settings = Settings {
    mouse_sensitivity: 0.30,
    chunk_render_distance: 5,
    move_speed: 0.1,
    near: 1.0,
    far: 1000.0,
};

pub struct SettingsWrapper;

impl Deref for SettingsWrapper {
    type Target = Settings;

    fn deref(&self) -> &Self::Target {
        unsafe {
            &SETTINGS_MUT
        }
    }
}

pub static SETTINGS: SettingsWrapper = SettingsWrapper;