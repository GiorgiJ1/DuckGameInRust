use macroquad::prelude::*;
use std::f32::consts::PI;

const SCREEN_WIDTH: f32 = 1000.0;
const SCREEN_HEIGHT: f32 = 750.0;

#[derive(PartialEq)]
enum EnemyType {
    Bug,
    MemoryLeak,
    NullPointer,
    StackOverflow,
}

#[derive(PartialEq)]
enum WeaponType {
    Debugger,
    CompilerCannon,
}

struct Player {
    pos: Vec2,
    speed: f32,
    hp: i32,
    max_hp: i32,
    weapon: WeaponType,
    shoot_cooldown: f32,
    rust_ownership_timer: f32,
    fire_rate_mod: f32,
    multishot: i32,
}

struct Bullet {
    pos: Vec2,
    vel: Vec2,
    damage: i32,
    size: f32,
    color: Color,
}

struct Enemy {
    pos: Vec2,
    enemy_type: EnemyType,
    speed: f32,
    hp: i32,
    max_hp: i32,
    size: f32,
    teleport_cooldown: f32,
}

struct Puddle {
    pos: Vec2,
    radius: f32,
    lifetime: f32,
}

struct MiniDuck {
    pos: Vec2,
    target_angle: f32,
}

struct PowerUp {
    pos: Vec2,
    kind: &'static str,
    size: f32,
}

struct Port {
    pos: Vec2,
    name: &'static str,
}

fn create_texture_from_matrix(sprite: &[&str], mapping: &[(char, (u8, u8, u8, u8))]) -> Texture2D {
    let mut bytes = vec![0u8; 16 * 16 * 4];
    for (y, row) in sprite.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if x >= 16 || y >= 16 { continue; }
            let idx = (y * 16 + x) * 4;
            let color = mapping.iter()
                .find(|(c, _)| *c == ch)
                .map(|(_, col)| *col)
                .unwrap_or((0, 0, 0, 0));
            bytes[idx] = color.0;
            bytes[idx + 1] = color.1;
            bytes[idx + 2] = color.2;
            bytes[idx + 3] = color.3;
        }
    }
    let texture = Texture2D::from_rgba8(16, 16, &bytes);
    texture.set_filter(FilterMode::Nearest);
    texture
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Skvanchi's Revenge: Kernel Defender".to_owned(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let duck_tex = create_texture_from_matrix(
        &[
            "....YYYY........", "...YYYYYY.......", "..YYYYBYY.......", "..YYYYYYY.......",
            "...OOOOO........", "....YYYYYYYY....", "..YYYYYYYYYYYY..", ".YYYYYYYYYYYYYY.",
            "YYYYYYYYYYYYYYYY", "YYYYYYYYYYYYYYYY", "YYYYYYYYYYYYYYYY", ".YYYYYYYYYYYYYY.",
            "..YYYYYYYYYYYY..", "...YYYYYYYYYY...", "....YY....YY....", "....OO....OO....",
        ],
        &[('Y', (255, 235, 59, 255)), ('O', (255, 152, 0, 255)), ('B', (0, 0, 0, 255))]
    );

    let bug_tex = create_texture_from_matrix(
        &[
            "M....GG....M", ".M...GG...M.", "..GGGGGGGG..", ".GGGGGGGGGG.",
            "GGGBGGGBGGGG", "GGGGGGGGGGGG", ".GGGGGGGGGG.", "..GGGGGGGG..",
            ".GGGGGGGGGG.", "G.GGGGGGGG.G", "G..G....G..G", "....M..M....",
        ],
        &[('G', (139, 195, 74, 255)), ('M', (244, 67, 54, 255)), ('B', (255, 255, 255, 255))]
    );

    let leak_tex = create_texture_from_matrix(
        &[
            "....PPPPPP....", "..PPPPPPPPPP..", ".PPPPPPPPPPPP.", "PPPPWPPPPWPPPP",
            "PPPPPPPPPPPPPP", "PPPPPPPPPPPPPP", ".PPPPPPPPPPPP.", "..PPPPPPPPPP..",
            "....PPPPPP....",
        ],
        &[('P', (156, 39, 176, 255)), ('W', (255, 255, 255, 255))]
    );

    let null_tex = create_texture_from_matrix(
        &[
            ".....CCCC.....", "...CC....CC...", "..CC..??..CC..", "..CC..??..CC..",
            "....??..??....", "......??......", "......??......", "......??......",
            "..............", "......??......", "......??......",
        ],
        &[('C', (0, 229, 255, 255)), ('?', (255, 255, 255, 255))]
    );

    let mut player = Player {
        pos: vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
        speed: 280.0,
        hp: 10,
        max_hp: 10,
        weapon: WeaponType::Debugger,
        shoot_cooldown: 0.0,
        rust_ownership_timer: 0.0,
        fire_rate_mod: 1.0,
        multishot: 1,
    };

    let ports = vec![
        Port { pos: vec2(100.0, 50.0), name: "USB PORT 1" },
        Port { pos: vec2(300.0, 50.0), name: "USB PORT 2" },
        Port { pos: vec2(600.0, 50.0), name: "ETHERNET" },
        Port { pos: vec2(900.0, 400.0), name: "CORRUPTED SECTOR" },
    ];

    let mut bullets: Vec<Bullet> = Vec::new();
    let mut enemies: Vec<Enemy> = Vec::new();
    let mut puddles: Vec<Puddle> = Vec::new();
    let mut mini_ducks: Vec<MiniDuck> = Vec::new();
    let mut powerups: Vec<PowerUp> = Vec::new();

    let mut wave = 1;
    let mut score = 0;
    let mut game_over = false;
    let mut is_bsod = false;
    let mut bsod_timer = 0.0;

    let mut spawn_system_wave = |w: i32, e_vec: &mut Vec<Enemy>, p_list: &Vec<Port>| {
        if w % 5 == 0 {
            e_vec.push(Enemy {
                pos: vec2(SCREEN_WIDTH / 2.0, 150.0),
                enemy_type: EnemyType::StackOverflow,
                speed: 60.0,
                hp: 50 + (w * 10),
                max_hp: 50 + (w * 10),
                size: 100.0,
                teleport_cooldown: 0.0,
            });
        } else {
            let num_enemies = 5 + w * 2;
            for _ in 0..num_enemies {
                let port = &p_list[rand::gen_range(0, p_list.len())];
                let spawn_offset = vec2(rand::gen_range(-20.0, 20.0), rand::gen_range(-20.0, 20.0));
                let etype = match rand::gen_range(0, 3) {
                    0 => EnemyType::Bug,
                    1 => EnemyType::MemoryLeak,
                    _ => EnemyType::NullPointer,
                };
                let (hp, speed, size) = match etype {
                    EnemyType::Bug => (2, 120.0, 30.0),
                    EnemyType::MemoryLeak => (4, 80.0, 35.0),
                    EnemyType::NullPointer => (1, 150.0, 25.0),
                    _ => (2, 100.0, 30.0),
                };
                e_vec.push(Enemy {
                    pos: port.pos + spawn_offset,
                    enemy_type: etype,
                    speed,
                    hp,
                    max_hp: hp,
                    size,
                    teleport_cooldown: 2.0,
                });
            }
        }
    };

    spawn_system_wave(wave, &mut enemies, &ports);

    loop {
        let delta = get_frame_time();

        if !game_over {
            if is_bsod {
                bsod_timer -= delta;
                if bsod_timer <= 0.0 {
                    is_bsod = false;
                }
            }

            if player.rust_ownership_timer > 0.0 {
                player.rust_ownership_timer -= delta;
            }
            if player.shoot_cooldown > 0.0 {
                player.shoot_cooldown -= delta;
            }

            let mut move_dir = Vec2::ZERO;
            if is_key_down(KeyCode::W) { move_dir.y -= 1.0; }
            if is_key_down(KeyCode::S) { move_dir.y += 1.0; }
            if is_key_down(KeyCode::A) { move_dir.x -= 1.0; }
            if is_key_down(KeyCode::D) { move_dir.x += 1.0; }
            
            if move_dir != Vec2::ZERO {
                player.pos += move_dir.normalize() * player.speed * delta;
            }

            player.pos.x = player.pos.x.clamp(20.0, SCREEN_WIDTH - 20.0);
            player.pos.y = player.pos.y.clamp(20.0, SCREEN_HEIGHT - 20.0);

            if is_key_pressed(KeyCode::Key1) { player.weapon = WeaponType::Debugger; }
            if is_key_pressed(KeyCode::Key2) { player.weapon = WeaponType::CompilerCannon; }

            let (mouse_x, mouse_y) = mouse_position();
            let mouse_pos = vec2(mouse_x, mouse_y);
            let look_dir = (mouse_pos - player.pos).normalize();

            if is_mouse_button_down(MouseButton::Left) && player.shoot_cooldown <= 0.0 {
                let base_angle = look_dir.y.atan2(look_dir.x);
                
                for i in 0..player.multishot {
                    let spread_angle = base_angle + (i as f32 - (player.multishot - 1) as f32 / 2.0) * 0.15;
                    let fire_vel = vec2(spread_angle.cos(), spread_angle.sin());

                    match player.weapon {
                        WeaponType::Debugger => {
                            bullets.push(Bullet {
                                pos: player.pos,
                                vel: fire_vel * 500.0,
                                damage: 1,
                                size: 6.0,
                                color: GREEN,
                            });
                            player.shoot_cooldown = 0.2 / player.fire_rate_mod;
                        }
                        WeaponType::CompilerCannon => {
                            bullets.push(Bullet {
                                pos: player.pos,
                                vel: fire_vel * 300.0,
                                damage: 5,
                                size: 18.0,
                                color: ORANGE,
                            });
                            player.shoot_cooldown = 0.6 / player.fire_rate_mod;
                        }
                    }
                }
            }

            for bullet in &mut bullets { bullet.pos += bullet.vel * delta; }
            bullets.retain(|b| b.pos.x > 0.0 && b.pos.x < SCREEN_WIDTH && b.pos.y > 0.0 && b.pos.y < SCREEN_HEIGHT);

            for puddle in &mut puddles { puddle.lifetime -= delta; }
            puddles.retain(|p| p.lifetime > 0.0);

            if is_key_pressed(KeyCode::E) && score >= 500 {
                score -= 500;
                for i in 0..4 {
                    mini_ducks.push(MiniDuck {
                        pos: player.pos,
                        target_angle: (i as f32) * (PI / 2.0),
                    });
                }
            }

            for (i, duck) in mini_ducks.iter_mut().enumerate() {
                duck.target_angle += 2.0 * delta;
                let offset = vec2(duck.target_angle.cos(), duck.target_angle.sin()) * 60.0;
                duck.pos = player.pos + offset;

                if i % 2 == 0 && rand::gen_range(0, 30) == 1 {
                    if let Some(enemy) = enemies.first() {
                        bullets.push(Bullet {
                            pos: duck.pos,
                            vel: (enemy.pos - duck.pos).normalize() * 400.0,
                            damage: 1,
                            size: 4.0,
                            color: YELLOW,
                        });
                    }
                }
            }

            for enemy in &mut enemies {
                let to_player = player.pos - enemy.pos;
                let dir = to_player.normalize();

                match enemy.enemy_type {
                    EnemyType::Bug => {
                        enemy.pos += dir * enemy.speed * delta;
                    }
                    EnemyType::MemoryLeak => {
                        enemy.pos += dir * enemy.speed * delta;
                        if rand::gen_range(0, 100) == 1 {
                            puddles.push(Puddle {
                                pos: enemy.pos,
                                radius: 25.0,
                                lifetime: 4.0,
                            });
                        }
                    }
                    EnemyType::NullPointer => {
                        enemy.teleport_cooldown -= delta;
                        if enemy.teleport_cooldown <= 0.0 {
                            enemy.pos = player.pos + vec2(rand::gen_range(-200.0, 200.0), rand::gen_range(-200.0, 200.0));
                            enemy.teleport_cooldown = rand::gen_range(2.0, 4.0);
                        }
                        enemy.pos += dir * enemy.speed * delta;
                    }
                    EnemyType::StackOverflow => {
                        enemy.pos += dir * enemy.speed * delta;
                        if rand::gen_range(0, 40) == 1 {
                            for a in 0..8 {
                                let angle = (a as f32) * (PI / 4.0);
                                bullets.push(Bullet {
                                    pos: enemy.pos,
                                    vel: vec2(angle.cos(), angle.sin()) * 200.0,
                                    damage: 2,
                                    size: 10.0,
                                    color: RED,
                                });
                            }
                        }
                    }
                }

                if to_player.length() < (enemy.size / 2.0 + 15.0) {
                    if player.rust_ownership_timer <= 0.0 {
                        player.hp -= 1;
                        player.rust_ownership_timer = 0.5;
                        if player.hp <= 0 { game_over = true; }
                    }
                }
            }

            for puddle in &puddles {
                if (player.pos - puddle.pos).length() < puddle.radius && player.rust_ownership_timer <= 0.0 {
                    player.hp -= 1;
                    player.rust_ownership_timer = 0.4;
                    if player.hp <= 0 { game_over = true; }
                }
            }

            for bullet in &mut bullets {
                for enemy in &mut enemies {
                    if enemy.hp > 0 && (bullet.pos - enemy.pos).length() < (enemy.size / 2.0 + bullet.size) {
                        enemy.hp -= bullet.damage;
                        bullet.pos = vec2(-999.0, -999.0);

                        if enemy.hp <= 0 {
                            score += match enemy.enemy_type {
                                EnemyType::StackOverflow => {
                                    is_bsod = true; 
                                    bsod_timer = 3.0; 
                                    2000
                                },
                                _ => 100,
                            };

                            if rand::gen_range(0, 10) < 3 {
                                let kinds = vec!["RAM Upgrade", "SSD Boost", "Fiber Internet", "Rust Ownership"];
                                powerups.push(PowerUp {
                                    pos: enemy.pos,
                                    kind: kinds[rand::gen_range(0, kinds.len())],
                                    size: 20.0,
                                });
                            }
                        }
                    }
                }
            }
            enemies.retain(|e| e.hp > 0);

            powerups.retain(|pu| {
                let dist = (player.pos - pu.pos).length();
                let collected = dist < (pu.size + 20.0);
                if collected {
                    match pu.kind {
                        "RAM Upgrade" => player.multishot += 1,
                        "SSD Boost" => player.speed += 50.0,
                        "Fiber Internet" => player.fire_rate_mod += 0.4,
                        "Rust Ownership" => player.rust_ownership_timer = 5.0,
                        _ => {}
                    }
                }
                !collected
            });

            if enemies.is_empty() && !is_bsod {
                wave += 1;
                spawn_system_wave(wave, &mut enemies, &ports);
                if player.hp < player.max_hp { player.hp += 1; }
            }

        } else if is_key_down(KeyCode::R) {
            player.pos = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
            player.hp = 10;
            player.multishot = 1;
            player.fire_rate_mod = 1.0;
            player.speed = 280.0;
            wave = 1;
            score = 0;
            enemies.clear();
            bullets.clear();
            powerups.clear();
            puddles.clear();
            mini_ducks.clear();
            is_bsod = false;
            spawn_system_wave(wave, &mut enemies, &ports);
            game_over = false;
        }

        clear_background(Color::from_rgba(10, 24, 16, 255));

        for x in (0..(SCREEN_WIDTH as i32)).step_by(80) {
            draw_line(x as f32, 0.0, x as f32, SCREEN_HEIGHT, 1.0, Color::from_rgba(20, 50, 30, 255));
        }
        for y in (0..(SCREEN_HEIGHT as i32)).step_by(80) {
            draw_line(0.0, y as f32, SCREEN_WIDTH, y as f32, 1.0, Color::from_rgba(20, 50, 30, 255));
        }

        for port in &ports {
            draw_rectangle(port.pos.x - 40.0, port.pos.y - 20.0, 80.0, 40.0, DARKGRAY);
            draw_rectangle_lines(port.pos.x - 40.0, port.pos.y - 20.0, 80.0, 40.0, 2.0, GREEN);
            draw_text(port.name, port.pos.x - 38.0, port.pos.y + 5.0, 10.0, WHITE);
        }

        for puddle in &puddles {
            draw_circle(puddle.pos.x, puddle.pos.y, puddle.radius, Color::from_rgba(156, 39, 176, 100));
        }

        for bullet in &bullets {
            draw_circle(bullet.pos.x, bullet.pos.y, bullet.size, bullet.color);
        }

        for pu in &powerups {
            draw_rectangle(pu.pos.x - 10.0, pu.pos.y - 10.0, 20.0, 20.0, GOLD);
            draw_text(pu.kind, pu.pos.x - 20.0, pu.pos.y - 15.0, 12.0, WHITE);
        }

        for duck in &mini_ducks {
            draw_texture_ex(&duck_tex, duck.pos.x - 10.0, duck.pos.y - 10.0, WHITE, DrawTextureParams { dest_size: Some(vec2(20.0, 20.0)), ..Default::default() });
        }

        for enemy in &enemies {
            let tex = match enemy.enemy_type {
                EnemyType::Bug => &bug_tex,
                EnemyType::MemoryLeak => &leak_tex,
                EnemyType::NullPointer => &null_tex,
                EnemyType::StackOverflow => &bug_tex,
            };

            if enemy.enemy_type == EnemyType::StackOverflow {
                draw_rectangle(enemy.pos.x - enemy.size / 2.0, enemy.pos.y - enemy.size / 2.0, enemy.size, enemy.size, RED);
                draw_text("██████████████", enemy.pos.x - 50.0, enemy.pos.y - 10.0, 20.0, BLACK);
                draw_text("█ OVERFLOW █", enemy.pos.x - 48.0, enemy.pos.y + 10.0, 16.0, WHITE);
                draw_text("██████████████", enemy.pos.x - 50.0, enemy.pos.y + 25.0, 20.0, BLACK);
            } else {
                draw_texture_ex(tex, enemy.pos.x - enemy.size / 2.0, enemy.pos.y - enemy.size / 2.0, WHITE, DrawTextureParams { dest_size: Some(vec2(enemy.size, enemy.size)), ..Default::default() });
            }

            let health_pct = enemy.hp as f32 / enemy.max_hp as f32;
            draw_rectangle(enemy.pos.x - enemy.size / 2.0, enemy.pos.y - enemy.size / 2.0 - 8.0, enemy.size, 4.0, RED);
            draw_rectangle(enemy.pos.x - enemy.size / 2.0, enemy.pos.y - enemy.size / 2.0 - 8.0, enemy.size * health_pct, 4.0, GREEN);
        }

        let (m_x, _) = mouse_position();
        let flip_x = m_x < player.pos.x;
        
        let color_tint = if player.rust_ownership_timer > 0.0 {
            Color::from_rgba(255, 100, 100, 255)
        } else {
            WHITE
        };

        draw_texture_ex(
            &duck_tex,
            player.pos.x - 20.0,
            player.pos.y - 20.0,
            color_tint,
            DrawTextureParams {
                dest_size: Some(vec2(40.0, 40.0)),
                flip_x,
                ..Default::default()
            },
        );

        draw_rectangle(0.0, 0.0, SCREEN_WIDTH, 40.0, Color::from_rgba(0, 0, 0, 200));
        draw_text(&format!("KERNEL HEALTH: {}/{}", player.hp, player.max_hp), 20.0, 26.0, 20.0, RED);
        draw_text(&format!("WAVE: {}", wave), 300.0, 26.0, 20.0, LIME);
        draw_text(&format!("DATACENTER SCORE: {}", score), 450.0, 26.0, 20.0, WHITE);
        draw_text(&format!("WEAPON: [1] Debugger [2] Compiler Cannon"), 700.0, 26.0, 15.0, Color::from_rgba(0, 255, 255, 255));
        if score >= 500 {
            draw_text("PRESS [E] FOR RUBBER DUCK MODE (Cost: 500)", SCREEN_WIDTH - 350.0, SCREEN_HEIGHT - 20.0, 15.0, GOLD);
        }

        if is_bsod {
            draw_rectangle(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, BLUE);
            draw_text(":(", 100.0, SCREEN_HEIGHT / 2.0 - 100.0, 100.0, WHITE);
            draw_text("Your system ran into a problem and needs to restart.", 100.0, SCREEN_HEIGHT / 2.0, 30.0, WHITE);
            draw_text("Flushing corrupted stack entries...", 100.0, SCREEN_HEIGHT / 2.0 + 50.0, 20.0, LIGHTGRAY);
        }

        if wave > 50 {
            draw_rectangle(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, Color::from_rgba(0, 40, 0, 230));
            draw_text("SERVER SECURED", SCREEN_WIDTH / 2.0 - 200.0, SCREEN_HEIGHT / 2.0 - 40.0, 50.0, LIME);
            draw_text("SKVANCHI SAVED THE DATACENTER", SCREEN_WIDTH / 2.0 - 260.0, SCREEN_HEIGHT / 2.0 + 20.0, 30.0, WHITE);
            draw_text("HOMELAB MODE UNLOCKED", SCREEN_WIDTH / 2.0 - 160.0, SCREEN_HEIGHT / 2.0 + 80.0, 22.0, GOLD);
        }

        if game_over {
            draw_rectangle(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, Color::from_rgba(20, 0, 0, 240));
            draw_text("CRITICAL_KERNEL_PANIC", SCREEN_WIDTH / 2.0 - 250.0, SCREEN_HEIGHT / 2.0 - 30.0, 40.0, RED);
            draw_text("Press 'R' to reload memory limits & try again", SCREEN_WIDTH / 2.0 - 220.0, SCREEN_HEIGHT / 2.0 + 30.0, 20.0, GRAY);
        }

        next_frame().await
    }
}