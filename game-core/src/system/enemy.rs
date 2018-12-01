use amethyst::core::cgmath::InnerSpace;
use amethyst::core::cgmath::Vector2;
use amethyst::{
    core::Transform,
    ecs::{Entities, Join, Read, ReadStorage, System, WriteStorage},
    renderer::{SpriteRender, Transparent},
};
use crate::component::{Animation, Enemy, Motion, Player};
use rand::distributions::{Distribution, Uniform};

pub struct Movement;

impl<'s> System<'s> for Movement {
    type SystemData = (
        ReadStorage<'s, Player>,
        ReadStorage<'s, Enemy>,
        WriteStorage<'s, Motion>,
        WriteStorage<'s, Transform>,
        Option<Read<'s, crate::map::PassableTiles>>,
    );

    fn run(&mut self, (players, enemies, mut motions, mut transforms, passable): Self::SystemData) {
        if let Some(passable) = passable {
            let mut player_translation = Vector2 { x: 0.0, y: 0.0 };
            let mut detection_circle = Vector2 { x: 64.0, y: 64.0};

            // get player position
            for (_, transform) in (&players, &mut transforms).join() {
                player_translation = transform.translation.truncate();
            }

            for (_, transform) in (&enemies, &mut transforms).join() {
                let enemy_translation = transform.translation.truncate();
                let player_direction = player_translation - enemy_translation;

                if player_direction.magnitude2() <= detection_circle.magnitude2() {
                    // let enemy_shift = player_direction - player_direction.normalize();
                    let enemy_shift = player_direction.normalize();
                    transform.translation += enemy_shift.extend(0.0);
                }
            }
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
                        .with(sprite, &mut sprites)
                        .with(Transparent, &mut transparent)
                        .with(anim, &mut animation)
                        .build();
                }
            }
        }
    }
}
