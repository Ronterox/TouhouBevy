use bevy::{app::AppExit, prelude::*};

#[derive(Component, Clone, Copy)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Enemy {}

#[derive(Component)]
struct Bullet {
    speed: f32,
}

#[derive(Resource)]
struct BulletTimer(Timer);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let player = Player { speed: 10. };
    let player_sprite = SpriteBundle {
        texture: asset_server.load("sakuya.png"),
        transform: Transform::from_xyz(0., -200., 0.).with_scale(Vec3::splat(0.2)),
        ..default()
    };
    commands.spawn((player, player_sprite));

    let enemy = Enemy {};
    let enemy_sprite = SpriteBundle {
        texture: asset_server.load("sakuya.png"),
        transform: Transform::from_xyz(0., 200., 0.).with_scale(Vec3::splat(0.15)),
        ..default()
    };
    commands.spawn((enemy, enemy_sprite));

    for _ in 0..5 {
        let bullet = Bullet { speed: 20. };
        let bullet_sprite = SpriteBundle {
            texture: asset_server.load("isaac.png"),
            transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.1)),
            visibility: Visibility::Hidden,
            ..default()
        };
        commands.spawn((bullet, bullet_sprite, player));
    }

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(80.),
                height: Val::Percent(80.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            background_color: Color::WHITE.into(),
            ..default()
        })
        .with_children(|panel| {
            panel
                .spawn(ButtonBundle {
                    background_color: Color::BLACK.into(),
                    style: Style {
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|button| {
                    button.spawn(TextBundle::from_section(
                        "Increment Speed",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
        });
}

fn move_by(transform: &mut Mut<Transform>, dir: (f32, f32), speed: f32) {
    transform.translation.x += dir.0 * speed;
    transform.translation.y += dir.1 * speed;
}

fn update_player_position(
    mut query_player: Query<(&Player, &mut Transform), Without<Bullet>>,
    keys: Res<ButtonInput<KeyCode>>,
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
}

fn update_bullet_shot(
    mut query_player: Query<&mut Transform, (With<Player>, Without<Bullet>)>,
    mut query_bullets: Query<
        (&mut Transform, &ViewVisibility, &mut Visibility),
        (With<Player>, With<Bullet>),
    >,
    time: Res<Time>,
    mut timer: ResMut<BulletTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        query_bullets
            .iter_mut()
            .find(|(_, is_visible, _)| !is_visible.get())
            .map(|(mut transform, _, mut visibility)| {
                transform.translation = query_player.single_mut().translation.clone();
                *visibility = Visibility::Visible;
            });
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

fn change_colors(
    mut enemies: Query<&mut Sprite, With<Enemy>>,
    mut bullets: Query<&mut Sprite, (With<Bullet>, Without<Enemy>)>,
) {
    enemies
        .iter_mut()
        .for_each(|mut sprite| sprite.color = Color::BLACK);
    bullets
        .iter_mut()
        .for_each(|mut sprite| sprite.color = Color::RED);
}

fn hide_ui(mut query: Query<&mut Visibility, With<Node>>) {
    for mut visibility in &mut query {
        *visibility = Visibility::Hidden;
    }
}

fn toggle_ui(
    mut query: Query<(&mut Visibility, &mut ViewVisibility), With<Node>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for (mut visibility, view_visibility) in &mut query {
            *visibility = match view_visibility.get() {
                true => Visibility::Hidden,
                false => Visibility::Visible,
            };
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(BulletTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .add_systems(Startup, startup)
        .add_systems(PostStartup, (change_colors, hide_ui))
        .add_systems(PreUpdate, (update_player_position, escape_game, toggle_ui))
        .add_systems(Update, (update_bullet_shot, update_bullets))
        .run();
}
