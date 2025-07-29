use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FrameTimeDiagnosticsPlugin::default(), EguiPlugin::default()))
            .add_systems(EguiPrimaryContextPass, (egui_menu,));
    }
}

fn egui_menu(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    window: Single<&mut Window>,
) -> Result {
    let fps_text = match diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        None => "N/A".into(),
        Some(value) => format!("{value:>4.0}"),
    };

    egui::Window::new("Debug")
        .resizable(true)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label("Left click to grab mouse, ESC to ungrab");
            ui.label(format!("FPS: {fps_text}"));
            ui.label(format!("VSync: {:?}", window.present_mode));
        });

    Ok(())
}
