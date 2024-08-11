use bevy::{
    app::AppExit,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Enemy {
    speed: f32,
    directions: Vec<f32>,
    move_timer: Timer,
}

#[derive(Component, Clone)]
struct Bullet {
    tag: Tag,
    direction: Vec3,
    speed: f32,
    damage: u32,
}

#[derive(Component)]
struct Gunner {
    tag: Tag,
    shot_timer: Timer,
}

#[derive(Component, Clone)]
struct ChangeColor(Color);

#[derive(Bundle, Clone)]
struct EntityBundle<T: Component> {
    data: T,
    sprite: SpriteBundle,
}

#[derive(Bundle)]
struct GunnerBundle<T: Component> {
    gunner: Gunner,
    health_bar: HealthBar,
    entity: EntityBundle<T>,
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

type PlayerBundle = GunnerBundle<Player>;
type EnemyBundle = GunnerBundle<Enemy>;
type BulletBundle = EntityBundle<Bullet>;

impl Gunner {
    fn new(tag: Tag, delay: f32) -> Self {
        Self {
            tag,
            shot_timer: Timer::from_seconds(delay, TimerMode::Repeating),
        }
    }

    fn can_shoot(&mut self, delta: std::time::Duration) -> bool {
        self.shot_timer.tick(delta).just_finished()
    }
}

impl PlayerBundle {
    fn new(texture: Handle<Image>) -> Self {
        Self {
            entity: EntityBundle {
                data: Player { speed: 10.0 },
                sprite: SpriteBundle {
                    transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::splat(0.12)),
                    texture,
                    ..default()
                },
            },
            gunner: Gunner::new(Tag::Player, 0.1),
            health_bar: HealthBar {
                tag: Tag::Player,
                health: 1,
                on_death: || todo!("Player died, you lose!"),
                on_hit: |health| println!("Player health: {}", health),
                hitbox_size: 25.,
            },
        }
    }
}

impl EnemyBundle {
    fn new(texture: Handle<Image>) -> Self {
        Self {
            entity: EntityBundle {
                data: Enemy {
                    speed: 5.,
                    directions: vec![-1., 0., 1., 0., 1., 0., -1., 0.],
                    move_timer: Timer::from_seconds(1.5, TimerMode::Repeating),
                },
                sprite: SpriteBundle {
                    transform: Transform::from_xyz(0., 200., 0.).with_scale(Vec3::splat(0.2)),
                    texture,
                    ..default()
                },
            },
            gunner: Gunner::new(Tag::Enemy, 0.5),
            health_bar: HealthBar {
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
        }
    }
}

impl BulletBundle {
    fn new(
        tag: Tag,
        texture: Handle<Image>,
        direction: Vec3,
        speed: f32,
        size_percentage: f32,
    ) -> Self {
        Self {
            data: Bullet {
                tag,
                speed,
                direction,
                damage: 1,
            },
            sprite: SpriteBundle {
                transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(size_percentage)),
                visibility: Visibility::Hidden,
                texture,
                ..default()
            },
        }
    }
}

impl HealthBar {
    fn take_damage(&mut self, damage: u32) {
        self.health -= damage;
        (self.on_hit)(self.health);
        if self.health == 0 {
            (self.on_death)();
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

    commands
        .spawn((PlayerBundle::new(asset_server.load(PLAYER_IMG_PATH)),))
        .with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle { radius: 50. })),
                material: materials.add(Color::rgba(1., 0., 0., 0.8)),
                transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                ..default()
            });
        });

    commands.spawn((
        EnemyBundle::new(asset_server.load(ENEMY_IMG_PATH)),
        ChangeColor(Color::BLACK),
    ));

    let bullet_texture = asset_server.load(BULLET_IMG_PATH);

    let bullet = BulletBundle::new(Tag::Player, bullet_texture.clone(), DIR_UP, 20., 0.05);
    commands.spawn_batch(std::iter::repeat((bullet, ChangeColor(Color::GRAY))).take(5));

    let bullet = BulletBundle::new(Tag::Enemy, bullet_texture, DIR_DOWN, 2., 0.1);
    commands.spawn_batch(std::iter::repeat((bullet, ChangeColor(Color::RED))).take(5));
}

fn update_player_position(
    mut player: Query<(&mut Transform, &Player)>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (mut ply_transform, ply_data) = player.single_mut();

    let mut if_press_move = |key: KeyCode, dir: Vec3| {
        if keys.pressed(key) {
            ply_transform.translation += dir * ply_data.speed;
        }
    };

    if_press_move(KeyCode::KeyW, DIR_UP);
    if_press_move(KeyCode::KeyA, DIR_LEFT);
    if_press_move(KeyCode::KeyS, DIR_DOWN);
    if_press_move(KeyCode::KeyD, DIR_RIGHT);
}

fn update_bullets_spawn(
    mut bullets: Query<(&mut Transform, &ViewVisibility, &mut Visibility, &Bullet)>,
    mut gunners: Query<(&Transform, &mut Gunner), Without<Bullet>>,
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

fn update_bullets_position(mut bullets: Query<(&Bullet, &mut Transform, &ViewVisibility)>) {
    bullets
        .iter_mut()
        .filter(|(_, _, visibility)| visibility.get())
        .for_each(|(bullet, mut transform, _)| {
            transform.translation += bullet.direction * bullet.speed;
        });
}

fn update_ai_position(mut enemies: Query<(&mut Enemy, &mut Transform)>, time: Res<Time>) {
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

fn update_entities_hits(
    mut entities: Query<(&Transform, &mut HealthBar)>,
    mut bullets: Query<(&Transform, &Bullet, &mut Visibility)>,
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
                        entity.take_damage(bullet.damage);
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
        .add_systems(PostUpdate, update_entities_hits)
        .run();
}
