use avian3d::prelude::PhysicsGizmos;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::*;
use bevy_shatter::Glass;

use crate::DisplayShards;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FrameTimeDiagnosticsPlugin::default(), EguiPlugin::default()))
            .add_systems(EguiPrimaryContextPass, egui_menu);
    }
}

fn egui_menu(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    window: Single<&mut Window>,
    mut config_store: ResMut<GizmoConfigStore>,
    glass: Single<Entity, With<Glass>>,
    mut commands: Commands,
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
            ui.label("Point at the glass, then use right click to shatter it");
            ui.label(format!("FPS: {fps_text}"));
            ui.label(format!("VSync: {:?}", window.present_mode));
            // FIX: clicking in the egui menu will hide it because of the spectator plugin, idk how to prevent this
            ui.checkbox(
                &mut config_store.config_mut::<PhysicsGizmos>().0.enabled,
                "Draw physics",
            );
            if ui.button("Shatter").clicked() {
                let glass_entity = glass.into_inner();
                commands.entity(glass_entity).insert(DisplayShards);
            }
        });

    Ok(())
}
