use bevy::prelude::*;

fn startup(mut commands: Commands) {
    let text_style = TextStyle {
        font_size: 30.0,
        ..Default::default()
    };

    commands.spawn(Camera2dBundle::default());
    commands.spawn(Text2dBundle {
        text: Text::from_section("Delta Time: ", text_style),
        ..default()
    });
}

fn update_text(time: Res<Time>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Delta Time: {:?}", time.delta());
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, update_text)
        .run();
}
