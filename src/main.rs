use bevy::{app::AppExit, prelude::*};

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("sakuya.png"),
        transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.2)),
        ..default()
    });
}

fn update_player(mut query: Query<(&mut Transform, &Sprite)>, keys: Res<ButtonInput<KeyCode>>) {
    let speed = 5.0;

    for (mut transform, _) in query.iter_mut() {
        let mut if_press_move = |key: KeyCode, dir: (f32, f32)| {
            if keys.pressed(key) {
                transform.translation.x += dir.0 * speed;
                transform.translation.y += dir.1 * speed;
            }
        };

        if_press_move(KeyCode::KeyW, (0., 1.));
        if_press_move(KeyCode::KeyA, (-1., 0.));
        if_press_move(KeyCode::KeyS, (0., -1.));
        if_press_move(KeyCode::KeyD, (1., 0.));
    }
}

fn escape_game(mut exit: EventWriter<AppExit>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(Update, (update_player, escape_game))
        .run();
}
