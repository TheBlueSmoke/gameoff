use amethyst::{
    core::cgmath::{InnerSpace, Vector2},
    core::timing::Time,
    core::Transform,
    ecs::{Entities, Join, Read, ReadStorage, System, WriteStorage},
    renderer::{SpriteRender, Transparent},
};
use crate::component::{Animation, Enemy, Motion, Player, Projectile};
use rand::distributions::{Distribution, Uniform};
use std::f32::consts::PI;
use std::time::Duration;

pub struct Movement {
    pub random_movement_time: Duration,
    pub random_idle_time: Duration,
}

impl<'s> System<'s> for Movement {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Enemy>,
        WriteStorage<'s, Motion>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (players, mut enemies, mut motions, transforms, time): Self::SystemData) {
        let idle_velocity = 50.0;
        let tracking_velocity = 100.0;

        let mut player_translation = Vector2 { x: 0.0, y: 0.0 };
        let detect_radius = 180.0;
        let detection_circle = Vector2 {
            x: detect_radius,
            y: detect_radius,
        };

        let time_accel = 4.0;
        // let current_second = (time.absolute_time_seconds() * time_accel).floor();

        // get player position
        for (_, transform) in (&players, &transforms).join() {
            player_translation = transform.translation.truncate();
        }

        for (enemy, motion, transform) in (&mut enemies, &mut motions, &transforms).join() {
            let enemy_translation = transform.translation.truncate();
            let player_direction = player_translation - enemy_translation;

            if player_direction.magnitude2() <= detection_circle.magnitude2() {
                // let enemy_shift = player_direction - player_direction.normalize();
                let enemy_shift = player_direction.normalize_to(tracking_velocity);
                motion.vel = enemy_shift;
                enemy.has_player_in_sight = true;
            } else {
                if motion.vel.magnitude2() > 0.0 {
                    if let Some(diff) = self.random_movement_time.checked_sub(time.delta_time()) {
                        self.random_movement_time = diff;
                    } else {
                        motion.vel = Vector2 { x: 0.0, y: 0.0 };
                        // entities.delete(entity);
                        self.random_idle_time = Duration::new(2, 0);
                    }
                }

                if motion.vel.magnitude2() == 0.0 {
                    if let Some(diff) = self.random_idle_time.checked_sub(time.delta_time()) {
                        self.random_idle_time = diff;
                    } else {
                        let range = Uniform::new_inclusive(0.0, 2.0 * PI);
                        let mut rng = rand::thread_rng();
                        let random_velocity = Vector2 {
                            x: range.sample(&mut rng).sin(),
                            y: range.sample(&mut rng).cos(),
                        };
                        motion.vel = random_velocity.normalize_to(idle_velocity);
                        // entities.delete(entity);
                        self.random_movement_time = Duration::new(2, 0);
                    }
                }

                /*
                if current_second % 2.0 == 0.0 {
                    if motion.vel.magnitude2() == 0.0 {
                        let range = Uniform::new_inclusive(0.0, 2.0 * PI);
                        let mut rng = rand::thread_rng();
                        let random_velocity = Vector2 {
                            x: range.sample(&mut rng).sin(),
                            y: range.sample(&mut rng).cos(),
                        };
                        motion.vel = random_velocity.normalize_to(idle_velocity);
                    }
                } else if current_second % 3.0 == 0.0 {
                    motion.vel = Vector2 { x: 0.0, y: 0.0 };
                }
                */
                enemy.has_player_in_sight = false;
            }
        }
    }
}

pub struct Attack;

impl<'s> System<'s> for Attack {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Enemy>,
        WriteStorage<'s, Transform>,
        Read<'s, crate::load::LoadedTextures>,
        WriteStorage<'s, Projectile>,
        WriteStorage<'s, Motion>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Transparent>,
        WriteStorage<'s, Animation>,
        Entities<'s>,
    );

    fn run(
        &mut self,
        (
            players,
            mut enemies,
            mut transforms,
            textures,
            mut projectiles,
            mut motions,
            mut sprites,
            mut transparent,
            mut animations,
            entities,
        ): Self::SystemData,
    ) {
        let mut bubble_transform = None;
        let mut bubble_dir = None;
        for (_player, _p_transform) in (&players, &transforms).join() {
            for (enemy, e_transform, e_motion) in (&mut enemies, &transforms, &motions).join() {
                // if they're moving they shoot
                if enemy.has_player_in_sight {
                    bubble_transform = Some(e_transform.clone());

                    let range = Uniform::new_inclusive(-5.0 * 32.0, 5.0 * 32.0);
                    let mut rng = rand::thread_rng();
                    let perp = e_motion.vel;
                    let perp = perp.normalize_to(range.sample(&mut rng));

                    bubble_dir = Some(e_motion.vel.normalize_to(32.0 * 23.0) + perp);
                }

                // do some dmg stuff here maybe
            }
        }

        if let Some(transform) = bubble_transform {
            let sprite = SpriteRender {
                sprite_sheet: textures.textures["bubble.png"].clone(),
                sprite_number: 0,
                flip_horizontal: false,
                flip_vertical: false,
            };

            let anim = Animation {
                total_frames: 2,
                max_count_till_next_frame: 0.5,
                frame_life_time_count: 0.5,
                current_frame: 0,
            };

            let motion = Motion {
                vel: bubble_dir.unwrap(),
                acc: bubble_dir.unwrap() * -2.0,
                min_vel: Some(32.0),
                max_vel: None,
            };

            entities
                .build_entity()
                .with(transform, &mut transforms)
                .with(Projectile, &mut projectiles)
                .with(motion, &mut motions)
                .with(sprite, &mut sprites)
                .with(Transparent, &mut transparent)
                .with(anim, &mut animations)
                .build();
        }
    }
}

pub struct Spawner;

impl<'s> System<'s> for Spawner {
    type SystemData = (
        ReadStorage<'s, Player>,
        Read<'s, crate::load::LoadedTextures>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Enemy>,
        WriteStorage<'s, Motion>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Transparent>,
        Entities<'s>,
        WriteStorage<'s, Animation>,
        Option<Read<'s, crate::map::PassableTiles>>,
    );

    fn run(
        &mut self,
        (
            players,
            textures,
            mut transforms,
            mut enemies,
            mut motions,
            mut sprites,
            mut transparent,
            entities,
            mut animation,
            passable,
        ): Self::SystemData,
    ) {
        let count = (&enemies).join().count();

        if let Some(passable) = passable {
            if count < 5 {
                let mut enemy_positions = vec![];
                let range = Uniform::new_inclusive(-5.0 * 32.0, 5.0 * 32.0);
                let mut rng = rand::thread_rng();
                for (_, transform) in (&players, &mut transforms).join() {
                    let mut pos = Transform::default();
                    pos.translation.x = transform.translation.x + range.sample(&mut rng);
                    pos.translation.y = transform.translation.y + range.sample(&mut rng);

                    // get tile and check if passable
                    let tile_y = (pos.translation.y as u32 / 32) as usize;
                    let tile_x = (pos.translation.x as u32 / 32) as usize;

                    if *passable
                        .tile_matrix
                        .get(tile_y)
                        .and_then(|row| row.get(tile_x))
                        .unwrap_or(&false)
                    {
                        enemy_positions.push(pos);
                    }
                }

                for pos in enemy_positions {
                    let sprite = SpriteRender {
                        sprite_sheet: textures.textures["penguinFront.png"].clone(),
                        sprite_number: 0,
                        flip_horizontal: false,
                        flip_vertical: false,
                    };

                    let anim = Animation {
                        total_frames: 2,
                        max_count_till_next_frame: 0.7,
                        frame_life_time_count: 0.7,
                        current_frame: 0,
                    };

                    entities
                        .build_entity()
                        .with(pos, &mut transforms)
                        .with(Enemy::default(), &mut enemies)
                        .with(Motion::default(), &mut motions)
                        .with(sprite, &mut sprites)
                        .with(Transparent, &mut transparent)
                        .with(anim, &mut animation)
                        .build();
                }
            }
        }
    }
}
