use super::DefeatedFort;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Door { Team0, Team1, Team2, Team3 }

impl Door {
    pub(crate) fn from_terrain(value: u8) -> Option<Self> {
        match value & 127 {
            20 | 68 => Some(Self::Team0),
            21 => Some(Self::Team1),
            60 => Some(Self::Team2),
            61 => Some(Self::Team3),
            _ => None,
        }
    }

    fn index(self) -> usize {
        match self { Self::Team0 => 0, Self::Team1 => 1, Self::Team2 => 2, Self::Team3 => 3 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    None,
    Defeat { fort: DefeatedFort, cause: i8 },
    ForcedMove { destination: [i16; 2] },
}

/// The terrain callback reads the UNIT owner bits. A hostile contact against
/// a pending team still requests admission; the admission gate rejects it.
/// That return cannot be redirected into the friendly/inactive door move.
pub(crate) fn contact(door: Door, flags31: u8, class: i8, position: [i16; 2], team_activity: [u8; 4], synthetic: bool) -> Action {
    if synthetic { return Action::None; }
    let owner = flags31 & 3;
    let index = door.index();
    if team_activity[index] != 0 && usize::from(owner) != index {
        let fort = match door {
            Door::Team0 => DefeatedFort::Team0,
            Door::Team1 => DefeatedFort::Team1,
            Door::Team2 | Door::Team3 => return Action::None,
        };
        return Action::Defeat { fort, cause: owner as i8 };
    }
    if class == 2 { return Action::None; }
    let dy = if matches!(door, Door::Team0 | Door::Team2) { -150 } else { 150 };
    Action::ForcedMove { destination: [position[0], position[1].wrapping_add(dy)] }
}

#[cfg(test)]
#[path = "original_fort_door_tests.rs"]
mod tests;
