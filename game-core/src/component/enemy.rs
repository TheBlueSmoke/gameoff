use amethyst::ecs::{Component, DenseVecStorage};

pub struct Enemy {
    pub hp: u32,
    pub has_player_in_sight: bool,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            hp: 120,
            has_player_in_sight: false,
        }
    }
}

impl Component for Enemy {
    type Storage = DenseVecStorage<Self>;
}
