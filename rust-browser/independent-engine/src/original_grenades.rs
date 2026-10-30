//! Confirmed grenade weapon choice and contact adapter, not a full projectile model.
//! GOG Z.exe 0044aa3a..0044aa7e gives Tough/Pyro/Laser their primary weapon first;
//! otherwise inventory+0x99 and target runtime class 1 or 2 select grenade state49.
//! 00464779 copies global0049ba74 (50) into projectile damage+0x1d. The blast
//! eventually uses body damage (0041f2d0 ->0041f260), without the bullet crew roll.
//! This adapter deliberately retains the engine's existing immediate target
//! contact and cooldown. Native 11-update throw windup, lob/scatter, allocation
//! pool, collision, splash geometry and original reacquisition remain unmodeled.
use super::{original_combat, Unit};

pub const CONTACT_DAMAGE: u8 = 50;

pub fn eligible(shooter: &str, inventory: usize, target: &str) -> bool {
    inventory > 0
        && matches!(shooter, "grunt" | "psycho" | "sniper")
        && matches!(original_combat::runtime_class(target), 1 | 2)
}

/// Call only after target acquisition, range, sight line and cooldown checks.
/// The current engine has no failing projectile allocator: committing this
/// contact is its successful-launch boundary. Failed/idle commands never call it.
pub fn try_throw(shooter: &mut Unit, target: &Unit) -> Option<original_combat::Attack> {
    if !eligible(&shooter.kind, shooter.grenades, &target.kind)
        || shooter.team < 0 || target.team < 0 || shooter.team == target.team
        || !original_combat::alive(true,shooter.hp) || !original_combat::alive(true,target.hp)
    {
        return None;
    }
    let mut attack=original_combat::Attack::from_unit(shooter,target.id,f64::from(CONTACT_DAMAGE),shooter.aim);
    attack.bullet=false;
    shooter.grenades-=1;
    Some(attack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Game, TYPES};

    fn engagement(shooter:&str,target:&str)->Game {
        let mut game=Game::new(0,1,371);
        game.map.original=true;
        game.map.tiles.fill(0);
        game.map.bridge_base.fill(-1);
        game.units.clear();game.buildings.clear();game.sectors.clear();
        game.objects.clear();game.crates.clear();game.effects.clear();game.events.clear();
        game.ai_teams.clear();game.ai_clock=100.;
        game.spawn(shooter,0,320.,320.,1,None);
        game.spawn(target,1,420.,320.,2,None);
        for unit in &mut game.units {unit.hold_position=true;}
        game.units[0].grenades=2;
        game.units[1].cooldown=100.;
        game
    }

    #[test]
    fn selection_matrix_matches_original_weapon_priority_and_target_classes() {
        for shooter in TYPES {
            for target in TYPES {
                let expected=matches!(shooter,"grunt"|"psycho"|"sniper")
                    && matches!(target,"cannon"|"medium"|"heavy"|"light"|"launcher"|"jeep"|"apc"|"crane"|"truck"|"firetruck");
                assert_eq!(eligible(shooter,1,target),expected,"{shooter} -> {target}");
                assert!(!eligible(shooter,0,target));
            }
        }
    }

    #[test]
    fn actual_original_shot_spends_one_grenade_for_fifty_hull_damage() {
        let mut game=engagement("grunt","medium");
        game.step(0.05);
        assert_eq!(game.units[0].grenades,1);
        assert_eq!(game.units[1].hp,150.);
        assert_eq!(game.units[1].crew.as_ref().unwrap().hp,30.);
        assert!(game.events.iter().any(|e|e["kind"]=="shot"&&e["weapon"]=="grenade"));
        game.step(0.05);
        assert_eq!(game.units[0].grenades,1,"Cooldown must not spend another grenade");
        assert_eq!(game.units[1].hp,150.);
    }

    #[test]
    fn empty_inventory_and_other_weapons_keep_their_native_attack() {
        for (shooter,target) in [("grunt","medium"),("tough","medium"),("pyro","medium"),("laser","medium"),("grunt","howitzer"),("grunt","gatling"),("grunt","missile")] {
            let mut game=engagement(shooter,target);
            if shooter=="grunt"&&target=="medium" {game.units[0].grenades=0;}
            let old=game.units[0].grenades;
            game.step(0.05);
            assert_eq!(game.units[0].grenades,old,"{shooter} -> {target}");
            assert!(game.events.iter().any(|e|e["kind"]=="shot"&&e["weapon"]=="primary"));
        }
    }

    #[test]
    fn uncommitted_attacks_do_not_consume_inventory() {
        for mode in ["cooldown","range","sight","friendly","neutral","moving"] {
            let mut game=engagement("grunt","medium");
            match mode {
                "cooldown"=>game.units[0].cooldown=100.,
                "range"=>game.units[1].x=800.,
                "sight"=>game.map.tiles[10*game.map.width+11]=2,
                "friendly"=>game.units[1].team=0,
                "neutral"=>{game.units[1].team=-1;game.units[1].crew=None;},
                "moving"=>{game.units[0].order="move".into();game.units[0].path.push_back((320.,700.));},
                _=>unreachable!(),
            }
            game.step(0.05);
            assert_eq!(game.units[0].grenades,2,"{mode}");
        }
    }

    #[test]
    fn procedural_attack_stays_on_its_existing_damage_path() {
        let mut game=engagement("grunt","medium");
        game.map.original=false;
        game.step(0.05);
        assert_eq!(game.units[0].grenades,1);
        assert_eq!(game.units[1].hp,157.);
        assert!(game.events.iter().any(|e|e["kind"]=="shot"&&e["weapon"]=="primary"));
    }

    #[test]
    fn howitzer_profile_is_stationary_artillery_instead_of_fallback_robot() {
        let state=original_combat::CombatState::new("howitzer");
        assert_eq!(original_combat::runtime_class("howitzer"),6);
        assert_eq!(state.aim_skill,60);
        assert_eq!(state.movement_control,0);
        assert_eq!(state.fire_rate,10);
        assert_eq!(original_combat::initial_crew_hp("howitzer"),30.);
        assert_eq!(state.reload_seconds(),90./17.);
        let mut game=engagement("grunt","howitzer");
        game.units[0].grenades=0;
        game.step(0.05);
        assert_eq!(game.units[1].hp,75.,"Grunt bullet damage5 becomes0 against artillery hull");
    }
}
