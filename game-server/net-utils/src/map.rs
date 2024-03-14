use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Point(pub i16, pub i16);
#[derive(Clone)]
pub enum GameDataType {
    Attack(Point),
    Movement(Point),
    Skip,
}
