use bevy::{app::AppExit, prelude::*};

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Bullet {
    speed: f32,
}

#[derive(Resource)]
struct BulletTimer(Timer);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let player = Player { speed: 5. };
    let player_sprite = SpriteBundle {
        texture: asset_server.load("sakuya.png"),
        transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.2)),
        ..default()
    };
    commands.spawn((player, player_sprite));

    // for i in 0..10 {
    //     let bullet = Bullet { speed: 5. };
    //     let bullet_sprite = SpriteBundle {
    //         texture: asset_server.load("isaac.png"),
    //         transform: Transform::from_xyz(0., 200. * i as f32, 0.).with_scale(Vec3::splat(0.2)),
    //         ..default()
    //     };
    //     commands.spawn((bullet, bullet_sprite));
    // }
}

fn move_by(transform: &mut Mut<Transform>, dir: (f32, f32), speed: f32) {
    transform.translation.x += dir.0 * speed;
    transform.translation.y += dir.1 * speed;
}

fn update_player(
    mut query: Query<(&Player, &mut Transform)>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut timer: ResMut<BulletTimer>,
    mut commands: Commands,
) {
    let (player, mut transform) = query.single_mut();

    let mut if_press_move = |key: KeyCode, dir: (f32, f32)| {
        if keys.pressed(key) {
            move_by(&mut transform, dir, player.speed);
        }
    };

    if_press_move(KeyCode::KeyW, (0., 1.));
    if_press_move(KeyCode::KeyA, (-1., 0.));
    if_press_move(KeyCode::KeyS, (0., -1.));
    if_press_move(KeyCode::KeyD, (1., 0.));

    if timer.0.tick(time.delta()).just_finished() {
        let bullet = Bullet { speed: 5. };
        let bullet_sprite = SpriteBundle {
            texture: asset_server.load("isaac.png"),
            transform: transform.clone().with_scale(Vec3::splat(0.1)),
            ..default()
        };
        commands.spawn((bullet, bullet_sprite));
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
        .insert_resource(BulletTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .add_systems(Startup, startup)
        .add_systems(Update, (update_player, escape_game, update_bullets))
        .run();
}
