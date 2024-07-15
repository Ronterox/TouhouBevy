use bevy::{app::AppExit, prelude::*};

#[derive(Component)]
struct PlayerData {
    speed: f32,
    shot_timer: Timer,
}

#[derive(Component, Clone)]
struct EnemyData {
    velocities: Vec<f32>,
    speed: f32,
    // shot_timer: Timer,
    move_timer: Timer,
}

#[derive(Component)]
struct BulletData {
    speed: f32,
}

#[derive(Bundle)]
struct BodyData<T: Component> {
    data: T,
    sprite: SpriteBundle,
}

type Player = BodyData<PlayerData>;
type Enemy = BodyData<EnemyData>;
type Bullet = BodyData<BulletData>;

impl Default for Player {
    fn default() -> Self {
        Self {
            data: PlayerData {
                speed: 10.0,
                shot_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            sprite: SpriteBundle {
                transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::splat(0.12)),
                ..default()
            },
        }
    }
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            data: EnemyData {
                velocities: vec![-1., 0., 1., 0., 1., 0., -1., 0.],
                speed: 5.,
                // shot_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                move_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            },
            sprite: SpriteBundle {
                transform: Transform::from_xyz(0., 200., 0.).with_scale(Vec3::splat(0.2)),
                ..default()
            },
        }
    }
}

impl Default for Bullet {
    fn default() -> Self {
        Self {
            data: BulletData { speed: 20. },
            sprite: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.05)),
                visibility: Visibility::Hidden,
                ..default()
            },
        }
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let mut player = Player::default();
    player.sprite.texture = asset_server.load("sakuya.png");
    commands.spawn(player);

    for _ in 0..5 {
        let mut bullet = Bullet::default();
        bullet.sprite.texture = asset_server.load("isaac.png");
        commands.spawn(bullet);
    }

    let mut enemy = Enemy::default();
    enemy.sprite.texture = asset_server.load("sakuya.png");
    commands.spawn(enemy);
}

fn move_by(transform: &mut Transform, dir: (f32, f32), speed: f32) {
    transform.translation.x += dir.0 * speed;
    transform.translation.y += dir.1 * speed;
}

fn update_player_position(
    mut player_pos: Query<(&mut Transform, &PlayerData)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (mut player_transform, player) = player_pos.single_mut();

    let mut if_press_move = |key: KeyCode, dir: (f32, f32)| {
        if keys.pressed(key) {
            move_by(&mut player_transform, dir, player.speed);
        }
    };

    if_press_move(KeyCode::KeyW, (0., 1.));
    if_press_move(KeyCode::KeyA, (-1., 0.));
    if_press_move(KeyCode::KeyS, (0., -1.));
    if_press_move(KeyCode::KeyD, (1., 0.));
}

fn update_bullet_spawn(
    mut query_bullets: Query<(&mut Transform, &ViewVisibility, &mut Visibility), With<BulletData>>,
    mut query_player: Query<(&Transform, &mut PlayerData), Without<BulletData>>,
    time: Res<Time>,
) {
    let (player_transform, mut player) = query_player.single_mut();

    if player.shot_timer.tick(time.delta()).just_finished() {
        query_bullets
            .iter_mut()
            .find(|(_, is_visible, _)| !is_visible.get())
            .map(|(mut transform, _, mut visibility)| {
                transform.translation = player_transform.translation;
                *visibility = Visibility::Visible;
            });
    }
}

fn update_bullets_position(
    mut query: Query<(&BulletData, &mut Transform, &ViewVisibility)>,
) {
    for (bullet, mut transform, visibility) in &mut query {
        if visibility.get() {
            transform.translation.y += bullet.speed;
        }
    }
}

fn update_enemy_position(
    mut query_enemy: Query<(&mut EnemyData, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut enemy, mut transform) in &mut query_enemy {
        if enemy.move_timer.tick(time.delta()).just_finished() {
            enemy.velocities.rotate_left(1);
        }
        let vel = enemy.velocities.first().unwrap_or(&0.);
        transform.translation.x += vel * enemy.speed;
    }
}

fn escape_game(mut exit: EventWriter<AppExit>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn change_colors(
    mut enemy_sprites: Query<&mut Sprite, (With<EnemyData>, Without<BulletData>)>,
    mut bullets: Query<&mut Sprite, With<BulletData>>,
) {
    enemy_sprites
        .iter_mut()
        .for_each(|mut sprite| sprite.color = Color::BLACK);

    bullets
        .iter_mut()
        .for_each(|mut sprite| sprite.color = Color::RED);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(PostStartup, change_colors)
        .add_systems(PreUpdate, (update_player_position, escape_game))
        .add_systems(
            Update,
            (
                update_bullet_spawn,
                update_bullets_position,
                update_enemy_position,
            ),
        )
        .run();
}
