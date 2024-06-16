use bevy::{app::AppExit, prelude::*};

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Bullet {
    speed: f32,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let player = Player { speed: 5. };
    let player_sprite = SpriteBundle {
        texture: asset_server.load("sakuya.png"),
        transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.2)),
        ..default()
    };
    commands.spawn((player, player_sprite));

    for i in 0..10 {
        let bullet = Bullet { speed: 5. };
        let bullet_sprite = SpriteBundle {
            texture: asset_server.load("isaac.png"),
            transform: Transform::from_xyz(0., 200. * i as f32, 0.).with_scale(Vec3::splat(0.2)),
            ..default()
        };
        commands.spawn((bullet, bullet_sprite));
    }
}

fn update_player(mut query: Query<(&Player, &mut Transform)>, keys: Res<ButtonInput<KeyCode>>) {
    for (player, mut transform) in &mut query {
        let mut if_press_move = |key: KeyCode, dir: (f32, f32)| {
            if keys.pressed(key) {
                transform.translation.x += dir.0 * player.speed;
                transform.translation.y += dir.1 * player.speed;
            }
        };

        if_press_move(KeyCode::KeyW, (0., 1.));
        if_press_move(KeyCode::KeyA, (-1., 0.));
        if_press_move(KeyCode::KeyS, (0., -1.));
        if_press_move(KeyCode::KeyD, (1., 0.));
    }
}

fn update_bullets(mut query: Query<(&Bullet, &mut Transform)>) {
    for (bullet, mut transform) in &mut query {
        transform.translation.y += bullet.speed;
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
        .add_systems(Update, (update_player, escape_game, update_bullets))
        .run();
}
