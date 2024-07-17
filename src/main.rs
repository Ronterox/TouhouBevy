use bevy::{
    app::AppExit,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

#[derive(Component)]
struct PlayerData {
    speed: f32,
    shot_timer: Timer,
}

#[derive(Component, Clone)]
struct EnemyData {
    velocities: Vec<f32>,
    speed: f32,
    shot_timer: Timer,
    move_timer: Timer,
    health: u32,
}

#[derive(Component, Clone)]
struct BulletData {
    speed: f32,
    direction: Vec2,
}

#[derive(Component)]
struct PlayerTag;

#[derive(Component)]
struct EnemyTag;

#[derive(Bundle, Clone)]
struct BodyData<T: Component> {
    data: T,
    sprite: SpriteBundle,
}

type Player = BodyData<PlayerData>;
type Enemy = BodyData<EnemyData>;
type Bullet = BodyData<BulletData>;

trait Shooter {
    fn get_timer(&mut self) -> &mut Timer;
    fn can_shoot(&mut self, delta: std::time::Duration) -> bool {
        self.get_timer().tick(delta).just_finished()
    }
}

impl Shooter for PlayerData {
    fn get_timer(&mut self) -> &mut Timer {
        &mut self.shot_timer
    }
}

impl Shooter for EnemyData {
    fn get_timer(&mut self) -> &mut Timer {
        &mut self.shot_timer
    }
}

impl Player {
    fn new(texture: &Handle<Image>) -> Self {
        Self {
            data: PlayerData {
                speed: 10.0,
                shot_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            },
            sprite: SpriteBundle {
                transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::splat(0.12)),
                texture: texture.clone(),
                ..default()
            },
        }
    }
}

impl Enemy {
    fn new(texture: &Handle<Image>) -> Self {
        Self {
            data: EnemyData {
                velocities: vec![-1., 0., 1., 0., 1., 0., -1., 0.],
                speed: 5.,
                shot_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                move_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
                health: 200,
            },
            sprite: SpriteBundle {
                transform: Transform::from_xyz(0., 200., 0.).with_scale(Vec3::splat(0.2)),
                texture: texture.clone(),
                ..default()
            },
        }
    }
}

impl Bullet {
    fn new(texture: &Handle<Image>, direction: [f32; 2], speed: f32, size_percentage: f32) -> Self {
        Self {
            data: BulletData {
                speed,
                direction: Vec2::from_array(direction)
            },
            sprite: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(size_percentage)),
                visibility: Visibility::Hidden,
                texture: texture.clone(),
                ..default()
            },
        }
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let sakuya_texture = asset_server.load("sakuya.png");
    commands.spawn(Player::new(&sakuya_texture));
    commands.spawn(Enemy::new(&sakuya_texture));

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle { radius: 10. })),
        material: materials.add(Color::rgba(1., 0., 0., 0.5)),
        transform: Transform::from_xyz(0., 0., 0.),
        ..default()
    });

    let bullet_texture = asset_server.load("isaac.png");

    let bullet = Bullet::new(&bullet_texture, [0., 1.], 20., 0.05);
    (0..5).for_each(|_| { commands.spawn((bullet.clone(), PlayerTag)); });

    let bullet = Bullet::new(&bullet_texture, [0., -1.], 2., 0.1);
    (0..5).for_each(|_| { commands.spawn((bullet.clone(), EnemyTag)); });
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

fn spawn_bullet<T: Shooter, K: Component, J: Component>(
    shooter: &mut Mut<T>,
    shooter_transform: &Transform,
    delta: std::time::Duration,
    bullets: &mut Query<
        (&mut Transform, &ViewVisibility, &mut Visibility),
        (With<BulletData>, With<K>, Without<J>),
    >,
) {
    if shooter.can_shoot(delta) {
        bullets
            .iter_mut()
            .find(|(_, is_visible, _)| !is_visible.get())
            .map(|(mut transform, _, mut visibility)| {
                transform.translation = shooter_transform.translation;
                *visibility = Visibility::Visible;
            });
    }
}

fn update_bullet_spawn(
    mut query_bullets_player: Query<
        (&mut Transform, &ViewVisibility, &mut Visibility),
        (With<BulletData>, With<PlayerTag>, Without<EnemyTag>),
    >,
    mut query_bullets_enemy: Query<
        (&mut Transform, &ViewVisibility, &mut Visibility),
        (With<BulletData>, With<EnemyTag>, Without<PlayerTag>),
    >,
    mut query_player: Query<(&Transform, &mut PlayerData), Without<BulletData>>,
    mut query_enemy: Query<(&Transform, &mut EnemyData), Without<BulletData>>,
    time: Res<Time>,
) {
    let (player_transform, mut player) = query_player.single_mut();
    let (enemy_transform, mut enemy) = query_enemy.single_mut();

    spawn_bullet(
        &mut player,
        player_transform,
        time.delta(),
        &mut query_bullets_player,
    );

    spawn_bullet(
        &mut enemy,
        enemy_transform,
        time.delta(),
        &mut query_bullets_enemy,
    );
}

fn update_bullets_position(mut query: Query<(&BulletData, &mut Transform, &ViewVisibility)>) {
    for (bullet, mut transform, visibility) in &mut query {
        if visibility.get() {
            transform.translation.x += bullet.speed * bullet.direction.x;
            transform.translation.y += bullet.speed * bullet.direction.y;
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

fn update_hits(
    query_player: Query<&Transform, (With<PlayerData>, Without<EnemyData>)>,
    mut query_enemy: Query<(&Transform, &mut EnemyData), Without<BulletData>>,
    mut query_hitbox: Query<
        &mut Transform,
        (
            With<Mesh2dHandle>,
            Without<PlayerData>,
            Without<EnemyData>,
            Without<BulletData>,
        ),
    >,
    query_enemy_bullets: Query<
        (&Transform, &Visibility),
        (With<BulletData>, (With<EnemyTag>, Without<PlayerTag>)),
    >,
    mut query_player_bullets: Query<
        (&Transform, &mut Visibility),
        (With<BulletData>, (With<PlayerTag>, Without<EnemyTag>)),
    >,
) {
    let player = query_player.single();
    let (enemy, mut enemy_data) = query_enemy.single_mut();

    // TODO: This should update on player move, use on changed system
    query_hitbox.single_mut().translation = player.translation;

    query_enemy_bullets
        .iter()
        .filter(|(_, visibility)| *visibility == Visibility::Visible)
        .for_each(|(bullet, _)| {
            if bullet.translation.distance(player.translation) < 25. {
                // TODO: Implement
                todo!("Player died, use size of bullet");
            }
        });

    query_player_bullets
        .iter_mut()
        .filter(|(_, visibility)| **visibility == Visibility::Visible)
        .for_each(|(bullet, mut visibility)| {
            if bullet.translation.distance(enemy.translation) < 100. {
                enemy_data.health -= 1;
                if enemy_data.health == 0 {
                    // TODO: Implement
                    todo!("Enemy died, You won!");
                }

                if enemy_data.health % 5 == 0 {
                    println!("Enemy health: {}", enemy_data.health);
                }

                *visibility = Visibility::Hidden;
            }
        });
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
        .add_systems(PostUpdate, update_hits)
        .run();
}
