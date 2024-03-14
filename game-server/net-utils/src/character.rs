pub enum CharacterClass {
    Bowman,
    Barbarian,
    Magician,
}

impl CharacterClass {
    pub fn new(class_str: &str) -> Option<Self> {
        Some(match class_str {
            "bar" => Self::Barbarian,
            "bow" => Self::Bowman,
            "mag" => Self::Magician,
            _ => return None,
        })
    }

    pub fn get_stats(&self) -> (u8, u8, u8, u8) {
        match self {
            Self::Bowman => (6, 70, 2, 6),
            Self::Barbarian => (10, 100, 4, 1),
            Self::Magician => (4, 80, 3, 2),
        }
    }
}

pub struct Character {
    //attack damage
    pub atk: u8,
    //health points
    pub hp: u8,
    //movement speed
    pub ms: u8,
    //attack range
    pub rng: u8,
    //character class
    pub class: CharacterClass,
}

impl Character {
    pub fn new(class: CharacterClass) -> Self {
        let stats = class.get_stats();
        Self {
            atk: stats.0,
            hp: stats.1,
            ms: stats.2,
            rng: stats.3,
            class,
        }
    }

    pub fn from_str(character_str: &str) -> Option<Character> {
        Some(match character_str {
            "bar" => Character::new(CharacterClass::Barbarian),
            "bow" => Character::new(CharacterClass::Bowman),
            "mag" => Character::new(CharacterClass::Magician),
            _ => return None,
        })
    }
}
