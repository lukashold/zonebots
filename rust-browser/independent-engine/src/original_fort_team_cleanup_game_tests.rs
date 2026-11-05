use super::super::{tests::field,State};
use crate::{original_death::Record,original_entry::HardwareAttributes,original_orders_game::{HardwareOrderRecord,HardwarePoolSlot}};
use serde_json::Value;

fn bytes(v:&Value)->Vec<u8>{v.as_str().unwrap().as_bytes().chunks_exact(2)
    .map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}
fn word(v:&[u8],i:usize)->i16{i16::from_le_bytes(v[i..i+2].try_into().unwrap())}
fn dword(v:&[u8],i:usize)->u32{u32::from_le_bytes(v[i..i+4].try_into().unwrap())}

#[test]
#[ignore="requires private original fort team retirement trace"]
fn fort_team_cleanup_retires_real_actors_and_preserves_usable_hardware() {
    let path=std::env::var("NATIVE_FORT_TEAM_TRACE").expect("private trace path");
    let source=std::fs::read_to_string(path).unwrap();
    let rows:Vec<Value>=source.lines().map(|line|serde_json::from_str(line).unwrap()).collect();
    let (mut objects,mut recoverable)=(0,0);
    assert_eq!(rows.len(),152);
    for (case,pair) in rows.chunks_exact(2).enumerate(){
        let (a,b)=(&pair[0],&pair[1]);assert_eq!(a["scenario"],case);assert_eq!(b["scenario"],case);
        let (before,after)=(bytes(&a["units"]),bytes(&b["units"]));
        let (hardware,expected_hardware)=(bytes(&a["hardware"]),bytes(&b["hardware"]));
        let mut g=field();let mut state=State::empty();
        // The source fixture supplies no free shared particle slot.
        state.effects.clear();state.fort_cleanup.count=a["teamCount"].as_i64().unwrap() as i16;
        state.fort_cleanup.marker=a["marker"].as_u64().unwrap() as u32;
        g.native_world=Some(state);g.native_display.fort_visible[1]=a["teamActive"].as_u64().unwrap() as u8;
        g.native_feedback.scene_gate=1;g.multiplayer=a["mp"]==1;
        for slot in 0..2 {
            let u=&before[slot*192..][..192];
            let kind=match u[48]{255=>"grunt",6=>"medium",25=>"heavy",24=>"launcher",30=>"light",34=>"jeep",32=>"apc",36=>"crane",family=>panic!("unexpected family {family}")};
            g.spawn(kind,1,f64::from(word(u,44))*2.,f64::from(word(u,46))*2.,slot+1,None);
            assert_eq!(g.units[slot].native_slot,Some(slot));
            g.units[slot].native_flags64=u[100];g.units[slot].native_class7c=u[124];
            let w=dword(u,8).checked_sub(0x59e6c2).filter(|&n|n<240*44);
            if let Some(w)=w {
                let w=w as usize;assert_eq!(w%44,0);
                let attrs=HardwareAttributes{bytes:hardware[w..w+44].try_into().unwrap()};
                g.native_hardware_pool[w/44]=HardwarePoolSlot{live_marker:word(&hardware,w),order:HardwareOrderRecord::from_bytes(&attrs.bytes),attributes:attrs};
                g.units[slot].native_hardware_slot=Some(w/44);
                // W, not the browser HP mirror, supplies the released hull.
                g.units[slot].hp=175.;
            }else{g.units[slot].native_hardware_slot=None;}
        }
        g.maintain_native_group_rosters();
        g.original_rng.words=std::array::from_fn(|i|a["rng"][i].as_u64().unwrap() as u32);
        g.remove_native_fort_team_actors(1).unwrap();
        assert_eq!(g.native_display.fort_visible[1],b["teamActive"].as_u64().unwrap() as u8,"visibility {case}");
        assert_eq!(g.native_fort_team_activity(),Some([255,255,0,0]));
        let state=g.native_world.as_ref().unwrap();
        assert_eq!(state.fort_cleanup.count,b["teamCount"].as_i64().unwrap() as i16,"count {case}");
        assert_eq!(state.fort_cleanup.marker,b["marker"].as_u64().unwrap() as u32,"marker {case}");
        let reentry=b["visits"].as_array().unwrap().iter().find(|v|v[0]==0x41ddb0).map(|v|v[1].as_u64().unwrap() as usize);
        assert_eq!(state.fort_cleanup.release_actor,reentry,"same actor {case}");
        for slot in 0..2 {
            assert_eq!(g.units[slot].native_slot,None,"retired browser binding {case}/{slot}");
            assert_eq!(g.units[slot].hp,-1.);
            assert_eq!(g.native_group_rosters.members[slot].as_ref().map_or(0,|m|m.flags31&4),after[slot*192+49]&4,"physical live bit {case}/{slot}");
            assert_eq!(g.native_group_rosters.contact_claim(slot),after[slot*192+181] as i8,"B5 {case}/{slot}");
            if let Some(w)=dword(&before,slot*192+8).checked_sub(0x59e6c2).filter(|&n|n<240*44) {
                let w=w as usize;
                assert_eq!(g.native_hardware_pool[w/44].attributes.bytes.as_slice(),&expected_hardware[w..w+44],"W {case}/{slot}");
                assert_eq!(g.units[slot].native_hardware_slot.is_none(),dword(&after,slot*192+8)==u32::MAX,"UNIT+8 {case}/{slot}");
            }
        }
        let records=bytes(&b["removals"]);
        for (i,r) in records.chunks_exact(8).enumerate(){
            let expected=Record{sprite:word(r,0),flags:word(r,2) as u16,position:[word(r,4),word(r,6)]};
            assert_eq!(g.native_deaths.records[i],expected,"corpse {case}/{i}");
        }
        assert_eq!(g.original_rng.words,std::array::from_fn(|i|b["rng"][i].as_u64().unwrap() as u32),"RNG {case}");
        let pool=bytes(&b["objects"]);
        for (slot,raw) in pool.chunks_exact(44).enumerate(){
            let object=state.objects[slot].as_ref().filter(|o|o.active);
            if word(raw,0)==-1{assert!(object.is_none(),"unexpected OO {case}/{slot}");continue;}
            let o=object.unwrap();objects+=1;
            assert_eq!(u32::from(o.kind),(dword(raw,4)-0x498f38)/26,"kind {case}/{slot}");
            assert_eq!(o.record.position,[word(raw,0),word(raw,2)],"OO position {case}/{slot}");
            assert_eq!([o.bucket12,o.record.flags13,o.hp,o.flags19,o.flags26,o.variant26 as u8,o.maximum],
                [raw[12],raw[13],raw[15],raw[19],raw[26],raw[38],raw[40]],"OO fields {case}/{slot}");
            assert_eq!(o.lifetime24,word(raw,36),"lifetime {case}/{slot}");
            for off in [14,16,17,24,25,27,28,29,30]{assert_eq!(o.hardware.unwrap().byte(off),raw[off],"OO payload {case}/{slot}/{off}");}
            if raw[19]==15 {
                let u=g.units.iter().find(|u|Some(u.id)==o.unit).unwrap();recoverable+=1;
                assert!(u.team<0&&u.crew.is_none()&&u.native_slot.is_none()&&u.native_hardware_slot.is_none());
                assert_eq!(u.hp,f64::from(raw[15])*25.);
            }else{assert!(o.unit.is_none());}
        }
        let rng=g.original_rng.words;let corpses=g.native_deaths.records;
        g.remove_native_fort_team_actors(1).unwrap();g.sync_native_world();
        assert_eq!(g.original_rng.words,rng,"already hidden {case}");assert_eq!(g.native_deaths.records,corpses);
        assert_eq!(g.native_world.as_ref().unwrap().objects.iter().flatten().filter(|o|o.active).count(),usize::from(word(&pool,0)!=-1));
    }
    assert_eq!((objects,recoverable),(28,7));
}
