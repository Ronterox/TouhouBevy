use bevy::{app::AppExit, prelude::*};

#[derive(Component, Clone, Copy)]
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

    let mut bullets = Vec::new();
    for _ in 0..5 {
        let bullet = Bullet { speed: 5. };
        let bullet_sprite = SpriteBundle {
            texture: asset_server.load("isaac.png"),
            transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.1)),
            visibility: Visibility::Hidden,
            ..default()
        };
        bullets.push((bullet, bullet_sprite, player));
    }
    commands.spawn((player, player_sprite));
    commands.spawn_batch(bullets);
}

fn move_by(transform: &mut Mut<Transform>, dir: (f32, f32), speed: f32) {
    transform.translation.x += dir.0 * speed;
    transform.translation.y += dir.1 * speed;
}

fn update_player(
    mut query_player: Query<(&Player, &mut Transform), Without<Bullet>>,
    mut query_bullets: Query<
        (&mut Transform, &ViewVisibility, &mut Visibility),
        (With<Player>, With<Bullet>),
    >,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut timer: ResMut<BulletTimer>,
) {
    let (player, mut player_transform) = query_player.single_mut();

    let mut if_press_move = |key: KeyCode, dir: (f32, f32)| {
        if keys.pressed(key) {
            move_by(&mut player_transform, dir, player.speed);
        }
    };

    if_press_move(KeyCode::KeyW, (0., 1.));
    if_press_move(KeyCode::KeyA, (-1., 0.));
    if_press_move(KeyCode::KeyS, (0., -1.));
    if_press_move(KeyCode::KeyD, (1., 0.));

    if timer.0.tick(time.delta()).just_finished() {
        for (mut transform, view_visibility, mut visibility) in &mut query_bullets {
            if !view_visibility.get() {
                transform.translation = player_transform.translation.clone();
                *visibility = Visibility::Visible;
                break;
            }
        }
    }
}

fn update_bullets(mut query: Query<(&Bullet, &mut Transform, &ViewVisibility)>) {
    for (bullet, mut transform, visibility) in &mut query {
        if visibility.get() {
            transform.translation.y += bullet.speed;
        }
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
