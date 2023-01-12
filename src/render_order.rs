use serde::{Deserialize, Serialize};

#[derive(Ord, PartialOrd, Eq, PartialEq, Deserialize, Serialize, Copy, Clone)]
pub enum RenderOrder {
    PlayerOrParticle,
    Monster,
    Item,
}
