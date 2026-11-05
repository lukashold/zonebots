use super::{contact, Action, Door};
use serde_json::Value;

#[test]
#[ignore = "requires private original door caller trace"]
fn original_fort_door_contact_matches_owner_class_and_forced_route_arguments() {
    let path = std::env::var("NATIVE_FORT_DOOR_TRACE").expect("private trace path");
    let text = std::fs::read_to_string(path).unwrap();
    let mut counts = [0;4];
    let rows: Vec<Value> = text.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    assert_eq!(rows.len(),512);
    for (id,row) in rows.iter().enumerate() {
        let n = |field: &str| row[field].as_i64().unwrap();
        assert_eq!(id as i64,n("id"));
        let door = n("door") as usize;
        let mut teams = [19,11,7,5];
        teams[door] = n("activity") as u8;
        let position = [n("x") as i16,n("y") as i16];
        let mut terrain = [vec![20,68],vec![21],vec![60],vec![61]][door].clone();
        terrain.extend(terrain.clone().into_iter().map(|value|value|128));
        for code in terrain {
            let result = contact(Door::from_terrain(code).unwrap(), n("owner") as u8 | 4,
                n("class") as i8, position, teams, n("synthetic") != 0);
            let (action,cause,forced,words,actor,opcode,flag) = match result {
                Action::None => (0,-1,-1,[0x11332244,0x55667788],0,0,0),
                Action::Defeat {fort,cause} => (i64::from(fort.team())+1,i64::from(cause),-1,[0x11332244,0x55667788],0,0,0),
                Action::ForcedMove {destination} => (3,-1,1,
                    [0x11330000|i64::from(destination[0] as u16),0x55660000|i64::from(destination[1] as u16)],
                    0x5e30c8+5*192,3,1),
            };
            assert_eq!([action,cause,forced,words[0],words[1],actor,opcode,flag,0],
                ["action","cause","forcedDuring","wordX","wordY","actor","opcode","routeFlag","forcedAfter"].map(n),"row {id} code {code}");
        }
        counts[n("action") as usize]+=1;
    }
    assert_eq!(counts,[356,36,36,84]);
}
