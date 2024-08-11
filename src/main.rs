use bevy::{
    app::AppExit,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

#[derive(Component)]
struct PlayerData {
    speed: f32,
}

#[derive(Component, Clone)]
struct EnemyData {
    speed: f32,
    directions: Vec<f32>,
    move_timer: Timer,
}

#[derive(Component, Clone)]
struct BulletData {
    tag: Tag,
    direction: Vec3,
    speed: f32,
    damage: u32,
}

#[derive(Component, Clone)]
struct Gunner {
    tag: Tag,
    shot_timer: Timer,
}

#[derive(Component, Clone)]
struct ChangeColor(Color);

#[derive(Bundle, Clone)]
struct BodyData<T: Component> {
    data: T,
    sprite: SpriteBundle,
}

#[derive(Component)]
struct HealthBar {
    tag: Tag,
    health: u32,
    on_death: fn(),
    on_hit: fn(health: u32),
    hitbox_size: f32,
}

#[derive(Debug, PartialEq, Clone)]
enum Tag {
    Player,
    Enemy,
}

type Player = BodyData<PlayerData>;
type Enemy = BodyData<EnemyData>;
type Bullet = BodyData<BulletData>;

impl Gunner {
    fn can_shoot(&mut self, delta: std::time::Duration) -> bool {
        self.shot_timer.tick(delta).just_finished()
    }
}

impl Player {
    fn new(texture: &Handle<Image>) -> Self {
        Self {
            data: PlayerData { speed: 10.0 },
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
                speed: 5.,
                directions: vec![-1., 0., 1., 0., 1., 0., -1., 0.],
                move_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
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
    fn new(
        tag: Tag,
        texture: &Handle<Image>,
        direction: Vec3,
        speed: f32,
        size_percentage: f32,
    ) -> Self {
        Self {
            data: BulletData {
                tag,
                speed,
                direction,
                damage: 1,
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

const BULLET_IMG_PATH: &str = "isaac.png";
const PLAYER_IMG_PATH: &str = "sakuya.png";
const ENEMY_IMG_PATH: &str = "sakuya.png";

const DIR_UP: Vec3 = Vec3::new(0., 1., 0.);
const DIR_DOWN: Vec3 = Vec3::new(0., -1., 0.);
const DIR_LEFT: Vec3 = Vec3::new(-1., 0., 0.);
const DIR_RIGHT: Vec3 = Vec3::new(1., 0., 0.);

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Player::new(&asset_server.load(PLAYER_IMG_PATH)),
        Gunner {
            tag: Tag::Player,
            shot_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        },
        HealthBar {
            tag: Tag::Player,
            health: 1,
            on_death: || todo!("Player died, you lose!"),
            on_hit: |health| println!("Player health: {}", health),
            hitbox_size: 25.,
        },
    ));

    commands.spawn((
        Enemy::new(&asset_server.load(ENEMY_IMG_PATH)),
        Gunner {
            tag: Tag::Enemy,
            shot_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        },
        ChangeColor(Color::BLACK),
        HealthBar {
            tag: Tag::Enemy,
            health: 200,
            on_death: || todo!("Enemy died, you win!"),
            on_hit: |health| {
                if health % 5 == 0 {
                    println!("Enemy health: {}", health);
                }
            },
            hitbox_size: 100.,
        },
    ));

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle { radius: 10. })),
        material: materials.add(Color::rgba(1., 0., 0., 0.5)),
        transform: Transform::from_xyz(0., -200., 0.),
        ..default()
    });

    let bullet_texture = asset_server.load(BULLET_IMG_PATH);
    let color = ChangeColor(Color::RED);

    let bullet = Bullet::new(Tag::Player, &bullet_texture, DIR_UP, 20., 0.05);
    commands.spawn_batch(std::iter::repeat((bullet, color.clone())).take(5));

    let bullet = Bullet::new(Tag::Enemy, &bullet_texture, DIR_DOWN, 2., 0.1);
    commands.spawn_batch(std::iter::repeat((bullet, color)).take(5));
}

fn update_player_position(
    mut player: Query<(&mut Transform, &PlayerData)>,
    mut hitbox: Query<&mut Transform, (With<Mesh2dHandle>, Without<PlayerData>)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (mut ply_transform, ply_data) = player.single_mut();

    let mut if_press_move = |key: KeyCode, dir: Vec3| {
        if keys.pressed(key) {
            ply_transform.translation += dir * ply_data.speed;
            hitbox.single_mut().translation = ply_transform.translation;
        }
    };

    if_press_move(KeyCode::KeyW, DIR_UP);
    if_press_move(KeyCode::KeyA, DIR_LEFT);
    if_press_move(KeyCode::KeyS, DIR_DOWN);
    if_press_move(KeyCode::KeyD, DIR_RIGHT);
}

fn update_bullets_spawn(
    mut bullets: Query<(
        &mut Transform,
        &ViewVisibility,
        &mut Visibility,
        &BulletData,
    )>,
    mut gunners: Query<(&Transform, &mut Gunner), Without<BulletData>>,
    time: Res<Time>,
) {
    for (gunner_transform, mut gunner) in &mut gunners {
        if !gunner.can_shoot(time.delta()) {
            continue;
        }

        bullets
            .iter_mut()
            .find(|(_, is_visible, _, data)| !is_visible.get() && data.tag == gunner.tag)
            .map(|(mut transform, _, mut visibility, _)| {
                transform.translation = gunner_transform.translation;
                *visibility = Visibility::Visible;
            });
    }
}

fn update_bullets_position(mut bullets: Query<(&BulletData, &mut Transform, &ViewVisibility)>) {
    bullets
        .iter_mut()
        .filter(|(_, _, visibility)| visibility.get())
        .for_each(|(bullet, mut transform, _)| {
            transform.translation += bullet.direction * bullet.speed;
        });
}

fn update_ai_position(mut enemies: Query<(&mut EnemyData, &mut Transform)>, time: Res<Time>) {
    for (mut enemy, mut transform) in &mut enemies {
        if enemy.move_timer.tick(time.delta()).just_finished() {
            enemy.directions.rotate_left(1);
        }
        let direction = enemy.directions.first().unwrap_or(&0.);
        transform.translation.x += direction * enemy.speed;
    }
}

fn escape_game(mut exit: EventWriter<AppExit>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn change_colors(mut sprites: Query<(&mut Sprite, &ChangeColor)>) {
    sprites
        .iter_mut()
        .for_each(|(mut sprite, ChangeColor(color))| sprite.color = *color);
}

fn update_entity_hit(
    mut entities: Query<(&Transform, &mut HealthBar)>,
    mut bullets: Query<(&Transform, &BulletData, &mut Visibility)>,
) {
    bullets
        .iter_mut()
        .filter(|(_, _, visibility)| **visibility == Visibility::Visible)
        .for_each(|(bullet_pos, bullet, mut bullet_visibility)| {
            entities
                .iter_mut()
                .filter(|(_, entity)| bullet.tag != entity.tag)
                .for_each(|(transform, mut entity)| {
                    if bullet_pos.translation.distance(transform.translation) < entity.hitbox_size {
                        entity.health -= bullet.damage;
                        (entity.on_hit)(entity.health);
                        if entity.health == 0 {
                            (entity.on_death)()
                        }
                        *bullet_visibility = Visibility::Hidden;
                    }
                });
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
                update_bullets_spawn,
                update_bullets_position,
                update_ai_position,
            ),
        )
        .add_systems(PostUpdate, update_entity_hit)
        .run();
}
