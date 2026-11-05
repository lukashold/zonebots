use super::{Bounds,Local,Move,disperse};
use crate::{Game,Value};

fn field()->Game {
    let mut game=Game::new(0,1,73);
    game.map.original=true;game.map.width=64;game.map.height=64;
    game.map.tiles=vec![0;4096];game.map.terrain_codes=vec![0;4096];game.map.native_terrain_plane=vec![0;16384];
    game.map.bridge_base=vec![-1;4096];game.map.original_tiles=vec![0;4096];
    game.units.clear();game.objects.clear();game.crates.clear();game.walls.clear();game.ai_teams.clear();
    game.native_group_rosters.clear();game.native_strategy=None;game.native_world=None;
    game.buildings.retain(|building|building.kind=="fort");
    assert_eq!(game.buildings.len(),2);
    for (i,building) in game.buildings.iter_mut().enumerate() {
        let (x,y)=(640.,640.+(i as f64)*160.);
        building.raw_base=Some((x,y));building.x=x*2.;building.y=y*2.;
    }
    game.import_native_world(&Value::Null);
    game
}

#[test]
fn fort_forced_move_uses_selected_offsets_without_broadcasting_to_other_groups() {
    let mut game=field();
    for i in 0..3 { game.spawn("grunt",0,360.+(i as f64)*32.,480.,11+i,None); }
    game.maintain_native_group_rosters();
    let groups:Vec<u8>=(11..14).map(|s|game.native_group_rosters.group_slot(0,s).unwrap() as u8).collect();
    assert!(game.select_native_groups(&groups).is_empty());
    game.native_selection.used[0]=1;
    let slot=game.units[1].native_slot.unwrap();
    game.native_selection.contexts[slot].icon_a8=Some(22);
    game.issue_native_fort_door_local_move(Move{slot:slot as u8,target:[200,300]}).unwrap();
    let m=game.native_group_rosters.members[slot].as_ref().unwrap();
    assert_eq!((m.promotion.target_x,m.promotion.target_y,m.promotion.target_aux),(184,300,3));
    assert_eq!(game.native_selection.contexts[slot].icon_a8,Some(0xabb));
    assert_eq!(&game.native_selection.used[..2],&[1,1]);
    assert_eq!(game.units.iter().map(|unit|unit.native_order5c).collect::<Vec<_>>(),[0,1,0]);
    assert_eq!(game.units[1].native_fieldb3,-1);
}

fn decode(value:&Value)->Vec<u8> {
    value.as_str().unwrap().as_bytes().chunks_exact(2)
        .map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()
}
fn word(row:&[u8],offset:usize)->i16 { i16::from_le_bytes(row[offset..offset+2].try_into().unwrap()) }
fn dword(row:&[u8],offset:usize)->i32 { i32::from_le_bytes(row[offset..offset+4].try_into().unwrap()) }

#[test]
#[ignore = "requires private original fort local orders trace"]
fn fort_local_game_router_matches_all_projected_unit_fields_group_rows_and_icons() {
    let path=std::env::var("NATIVE_FORT_SWEEP_TRACE").expect("private trace path");
    let text=std::fs::read_to_string(path).unwrap();
    let raw:Vec<Value>=text.lines().map(|s|serde_json::from_str(s).unwrap()).collect();
    assert_eq!(raw.len(),192);
    let mut cases=0;
    let mut routed=0;
    for (id,pair) in raw.chunks_exact(2).enumerate() {
        let (before,after)=(&pair[0],&pair[1]);
        let before_field=|n:usize|before["fields"][n].as_u64().unwrap();
        let after_field=|n:usize|after["fields"][n].as_u64().unwrap();
        if before_field(0)!=0&&before_field(1)==0 { continue; }
        cases+=1;
        routed+=after["visits"].as_array().unwrap().iter().filter(|r|r[0]==0x45f070).count();
        let mut game=field();
        game.spawn("grunt",0,360.,480.,1,None);
        game.maintain_native_group_rosters();
        let unit=game.units.pop().unwrap();
        let member=game.native_group_rosters.members[unit.native_slot.unwrap()].clone().unwrap();
        game.native_group_rosters.clear();
        let start_units=decode(&before["units"]);
        let end_units=decode(&after["units"]);
        let start_groups=decode(&before["groups"]);
        let end_groups=decode(&after["groups"]);
        for (slot,record) in start_units.chunks_exact(192).enumerate() {
            game.native_selection.contexts[slot].icon_a8=Some(word(record,168) as u16);
            if record[49]&4==0 { continue; }
            let mut u=unit.clone();
            u.id=slot+1;u.native_slot=Some(slot);u.team=i32::from(record[49]&3);
            u.x=f64::from(word(record,44))*2.;u.y=f64::from(word(record,46))*2.;
            u.native_class7c=record[124];u.native_order5c=record[92] as i8;
            u.native_counter6e=record[110];u.native_flags9c=record[156];u.native_fieldb3=record[179] as i8;
            let mut m=member.clone();
            m.slot=slot as u8;m.group=record[95];m.role=record[94] as i8;m.flags31=record[49];m.flags69=record[105];m.flags9c=record[156];
            m.promotion.order=record[92];m.promotion.target_x=dword(record,80);
            m.promotion.target_y=word(record,84);m.promotion.target_aux=word(record,86);
            game.native_group_rosters.members[slot]=Some(m);
            game.units.push(u);
        }
        for (g,bytes) in game.native_group_rosters.pool.iter_mut().zip(start_groups.chunks_exact(72)) {g.roster.copy_from_slice(bytes);}
        game.native_order_arguments=[before_field(15) as i16,before_field(16) as i16,before_field(13) as i16];
        let rect=|n:usize|before["rect"][n].as_i64().unwrap() as i32;
        let bounds=Bounds{left:rect(0),top:rect(1),right:rect(2),bottom:rect(3)};
        let mut world=Local{game:&mut game,bounds};
        disperse(&mut world,bounds,(id%4) as u16,before["direction"].as_u64().unwrap()!=0).unwrap();
        for u in &game.units {
            let slot=u.native_slot.unwrap();
            let m=game.native_group_rosters.members[slot].as_ref().unwrap();
            let end=&end_units[slot*192..(slot+1)*192];
            assert_eq!([u.native_order5c as u8,u.native_counter6e,u.native_flags9c,u.native_fieldb3 as u8,m.flags69],
                [92,110,156,179,105].map(|offset|end[offset]),"{id}/{slot}");
            assert_eq!((m.promotion.target_x,m.promotion.target_y,m.promotion.target_aux),
                (dword(end,80),word(end,84),word(end,86)),"targets {id}/{slot}");
        }
        for (slot,record) in end_units.chunks_exact(192).enumerate() {
            assert_eq!(game.native_selection.contexts[slot].icon_a8,Some(word(record,168) as u16),"icon {id}/{slot}");
        }
        for (slot,(group,record)) in game.native_group_rosters.pool.iter().zip(end_groups.chunks_exact(72)).enumerate() {
            assert_eq!(group.roster.as_slice(),record,"group {id}/{slot}");
        }
        assert_eq!(game.native_order_arguments,[after_field(15) as i16,after_field(16) as i16,after_field(13) as i16]);
    }
    assert_eq!((cases,routed),(72,378));
}
