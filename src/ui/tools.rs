#[macro_export]
macro_rules!  text_fmt {
($ui:expr, $($arg:tt)*) => {
    $ui.text(format!($($arg)*));
};
}

pub fn disabled<F>(ui: &imgui::Ui, func: F)
where
    F: FnOnce(),
{
    let _d = ui.begin_disabled(true);
    func();
}

pub fn set_dark_theme_colors(style: &mut imgui::Style) {
    const DARK_GREY: [f32; 4] = [0.1, 0.105, 0.11, 1.0];
    const COLD_GREY: [f32; 4] = [0.2, 0.205, 0.21, 1.0];
    const DARK_COLD_GREY: [f32; 4] = [0.15, 0.1505, 0.151, 1.0];
    const GREY: [f32; 4] = [0.28, 0.2805, 0.281, 1.0];
    const MEDIUM_GREY: [f32; 4] = [0.3, 0.305, 0.31, 1.0];
    const LIGHT_GREY: [f32; 4] = [0.38, 0.3805, 0.381, 1.0];

    // let DarkGrey: imgui::ImColor32 = imgui::ImColor32::from_rgb_f32s(0.1, 0.105, 0.11);

    let colors = &mut style.colors;

    colors[imgui::StyleColor::WindowBg as usize] = DARK_GREY;

    // Headers
    colors[imgui::StyleColor::Header as usize] = COLD_GREY;
    colors[imgui::StyleColor::HeaderHovered as usize] = MEDIUM_GREY;
    colors[imgui::StyleColor::HeaderActive as usize] = DARK_COLD_GREY;

    // Buttons
    colors[imgui::StyleColor::Button as usize] = COLD_GREY;
    colors[imgui::StyleColor::ButtonHovered as usize] = MEDIUM_GREY;
    colors[imgui::StyleColor::ButtonActive as usize] = DARK_COLD_GREY;

    // Frame BG
    colors[imgui::StyleColor::FrameBg as usize] = COLD_GREY;
    colors[imgui::StyleColor::FrameBgHovered as usize] = MEDIUM_GREY;
    colors[imgui::StyleColor::FrameBgActive as usize] = DARK_COLD_GREY;

    // Tabs
    colors[imgui::StyleColor::Tab as usize] = DARK_COLD_GREY;
    colors[imgui::StyleColor::TabHovered as usize] = LIGHT_GREY;
    colors[imgui::StyleColor::TabActive as usize] = GREY;
    colors[imgui::StyleColor::TabUnfocused as usize] = DARK_COLD_GREY;
    colors[imgui::StyleColor::TabUnfocusedActive as usize] = COLD_GREY;

    // Title
    colors[imgui::StyleColor::TitleBg as usize] = DARK_COLD_GREY;
    colors[imgui::StyleColor::TitleBgActive as usize] = DARK_COLD_GREY;
    colors[imgui::StyleColor::TitleBgCollapsed as usize] = DARK_COLD_GREY;
}
