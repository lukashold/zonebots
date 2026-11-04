//! World-state bindings for the verified contact kernels. An OBJECT pool slot
//! has one lifetime even when it is represented by a selectable neutral unit,
//! a pickup and a renderer record. Capturing a unit retires that OBJECT slot.
use super::{Game, Building, json, stats, original_combat::{self, Attack}, original_world_contact as native, native_ai_tasks as task};
use serde_json::Value;
use std::collections::{BTreeSet,VecDeque};
#[path="original_world_effects.rs"] mod effects;
#[path="original_building_trail.rs"]mod trail;
#[path="original_crane_building.rs"]mod crane;
#[path="original_crane_building_game.rs"]mod crane_game;
#[path="original_crane_bridge.rs"]mod crane_bridge;
#[path="original_crane_bridge_game.rs"]mod crane_bridge_game;
#[path="original_building_trail_game.rs"]mod trail_game;
#[path="original_building_blast_game.rs"]mod trail_blast;
#[path="original_world_fire.rs"] pub(crate) mod fire;
#[path="original_pickups.rs"] pub(crate) mod pickups;
#[path="original_workshop.rs"] pub(crate) mod workshop;
#[path="original_workshop_game.rs"] mod workshop_game;
#[path="original_building_fragments_game.rs"] mod building_fragments;
#[path="original_fort_fragments.rs"]mod fort_fragments;
#[path="original_fort_fragments_game.rs"]mod fort_fragments_game;
#[path="original_fragment_blast_game.rs"] mod fragment_blast;
#[cfg(test)]#[path="original_display_objects_game_tests.rs"]mod object_display_tests;
#[path="original_water_spray_game.rs"] mod water_spray;
#[path="original_bridge_sound_game.rs"] mod bridge_sound;
#[path="original_bridge_effects_game.rs"]mod bridge_effects;
#[path="original_seasonal_game.rs"]mod seasonal_game;

#[derive(Clone)]
struct ObjectSlot {
    active: bool, record: native::Object, hp: u8, maximum: u8, kind: u16, sprite: u16,extent21:i8,
    bucket12:u8,flags19:u8,flags26: u8,variant26:i8,lifetime24:i16,animation18:u8,sprite_offset21:i8,motion22:i8,hardware:Option<super::original_entry::ObjectHardware>,workshop_raw18_19:Option<[u8;2]>,workshop_raw1b:Option<u8>, flag_sector:Option<usize>,flag_delay1f:u8,saved: [u8;4], unit: Option<usize>, pickup: Option<usize>, render: Option<usize>, fire: Option<usize>,
}

#[derive(Clone)]
struct BuildingSlot { record: native::Building, maximum: i16, binding: Option<usize>, fort_proxy: bool, repair:Option<workshop::State>, constructor:Option<super::original_factory::ConstructorFields>,display:super::original_building_display::Radar }
#[derive(Clone)]
struct BridgeSlot { record: native::Bridge, binding: usize, native_slot:usize, orientation:i16,
    retired_road:Option<(usize,[i16;4])>, restored_terrain:Vec<(i16,u8)>,raw_tail:[u8;32],native_work_owned:bool }
#[derive(Clone, Copy)]
struct Fort { binding: usize, base: [i16;2] }
pub(crate) struct ScanObject{pub slot:u8,pub position:[i16;2],pub bucket12:u8,pub flags13:u8,pub flags19:u8,pub flags26:u8,pub definition:u16,pub sprite:u16,pub template_flags18:u8,pub extent20:i8,pub extent21:i8,pub unit:Option<usize>,pub flag_sector:Option<usize>,pub flag_delay1f:u8}

pub struct State {
    ///Raw4d18c8, written only by43ea80. A miss and a mission import preserve it.
    order_object_pointer:u32,
    trail:trail::State,trail_draws:Vec<Value>,direct_palette:Option<u8>,trail_main_result:Option<u8>,trail_scenes:[i32;4],cranes:Vec<Option<crane::Crane>>,crane_frames:Vec<Vec<[i16;4]>>,bridge_crane_frames:Vec<Vec<Vec<[i16;4]>>>,bridge_repair_ready:bool,ruin_stamps:std::collections::BTreeMap<i8,(u32,usize,usize,Vec<u8>)>,alive_stamps:std::collections::BTreeMap<i8,(u32,usize,usize,Vec<u8>)>,effect_world_suppression:u8,effect_world_mask4f:u8,effect_wall_mask43:u8,effect_quality:i8,
    objects: Vec<Option<ObjectSlot>>, buildings: Vec<BuildingSlot>, bridges: Vec<BridgeSlot>,retired_workshop_alias25:Option<super::original_retired_workshop::Alias>,retired_preamble_offset21_24:Option<i8>,
    forts: Vec<Fort>, suppress_objects: u8, effects:Vec<Option<effects::Particle>>, effect_cp_tints:Vec<Option<u8>>,effect_draws:Vec<(usize,effects::Draw,Option<u8>)>,effect_view:Option<([i16;2],[i16;4])>,effect_tint:u8,effect_owner:u8,effect_maximum:i8,effect_origin:[i16;2],effect_flags:u8,effect_fade:u8,effect_sprite:i16,effect_suppression:u8,bridge_fx:effects::bridge::Raw,last_projection:[i16;2],quiet_blasts:bool,fragments:building_fragments::State,fort_fragments:fort_fragments::State,seasonal_cps:Option<seasonal_game::CpPool>,
}
impl State {
    fn empty() -> Self { Self { order_object_pointer:0,trail:Default::default(),trail_draws:vec![],direct_palette:None,trail_main_result:None,trail_scenes:[-1;4],cranes:vec![None;240],crane_frames:vec![],bridge_crane_frames:vec![],bridge_repair_ready:false,ruin_stamps:Default::default(),alive_stamps:Default::default(),effect_world_suppression:0,effect_world_mask4f:0,effect_wall_mask43:0,effect_quality:0, objects: vec![None;200],retired_workshop_alias25:None,retired_preamble_offset21_24:None, buildings: Vec::new(), bridges: Vec::new(), forts: Vec::new(), suppress_objects:0,effects:vec![None;500],effect_cp_tints:vec![None;500],effect_draws:Vec::new(),effect_view:None,effect_tint:0,effect_owner:0,effect_maximum:0,effect_origin:[0,0],effect_flags:0,effect_fade:0,effect_sprite:0,effect_suppression:0,bridge_fx:Default::default(),last_projection:[0,0],quiet_blasts:false,fragments:Default::default(),fort_fragments:Default::default(),seasonal_cps:None } }
    fn free(&self) -> Option<usize> { self.objects.iter().position(|s|s.as_ref().is_none_or(|s|!s.active)) }
    ///43ecf0/43eee0 and58a40 do not write raw+0x26. Only58af0's
    ///destroyed-hardware branches replace this warm byte with RNG&3.
    fn put_object(&mut self,slot:usize,mut object:ObjectSlot){if slot==24{self.retired_preamble_offset21_24=Some(object.sprite_offset21);}let latent=if slot==25{self.retired_workshop_alias25.take()}else{None};object.variant26=self.objects[slot].as_ref().map_or_else(||latent.map_or(0,|a|a.byte26 as i8),|old|old.variant26);object.workshop_raw1b=object.hardware.map(|h|h.byte(27)).or(object.workshop_raw1b).or_else(||self.objects[slot].as_ref().and_then(|old|old.workshop_raw1b)).or_else(||latent.map(|a|a.word1a.to_le_bytes()[1])).or(Some(0));self.objects[slot]=Some(object);}
    ///Renderer provenance is not an invented Raw44 field. A real new
    ///construction clears it; only402c80's allocator sets a CP owner.
    fn put_effect(&mut self,slot:usize,particle:effects::Particle){self.effects[slot]=Some(particle);self.effect_cp_tints.resize(self.effects.len(),None);self.effect_cp_tints[slot]=None;}
}

fn number(v:&Value,key:&str,default:i64)->i64 { v[key].as_i64().unwrap_or(default) }
fn native_point(x:f64,y:f64)->[i16;2] { [(x/2.).floor() as i16,(y/2.).floor() as i16] }
fn native_bucket(p:[i16;2])->u8{native::initial_object_bucket(p)}
fn equipment_kind(eq:u16)->Option<&'static str> { ["cannon","medium","grunt","tough","sniper","heavy","launcher","psycho","light","missile","apc","gatling","howitzer","jeep","firetruck","crane","truck","pyro","laser"].get(usize::from(eq)).copied() }
fn object_kind(kind:&str)->Option<u16> { Some(match kind { "cannon"=>5,"medium"=>8,"missile"=>21,"gatling"=>22,"howitzer"=>23,"launcher"=>24,"heavy"=>26,"light"=>29,"apc"=>31,"jeep"=>33,"crane"=>35,"firetruck"=>37,_=>return None }) }
fn building_kind(kind:&str)->Option<i8> { Some(match kind { "repair"=>0,"vehicle"=>1,"fort"=>2,"robot"=>4,"radar"=>8,_=>return None }) }

impl Game {
    pub(crate) fn world_sector(&self,point:[i16;2])->Option<usize> {
        if let Some(s)=&self.native_strategy { return s.sector_at(f64::from(point[0])*2.,f64::from(point[1])*2.); }
        let(x,y)=(point[0]>>4,point[1]>>4);if x<0||y<0||x as usize>=self.map.width||y as usize>=self.map.height{return None;}
        let id=*self.map.territory.get(y as usize*self.map.width+x as usize)?;
        self.sectors.iter().find(|s|s.id==id&&s.counts_for_production).map(|s|s.id)
    }
    fn world_code(&self,cell:i16)->u8 {
        if cell<0{return 0;}let i=cell as usize;let(x,y)=(i&127,i>>7);
        if x<self.map.width&&y<self.map.height { (self.map.terrain_codes.get(y*self.map.width+x).copied().unwrap_or(0)&127)|(self.map.native_terrain_plane.get(i).copied().unwrap_or_else(||((self.map.original_tiles.get(y*self.map.width+x).copied().unwrap_or(0)&256)>>1)as u8)&128) }
        else { self.map.native_terrain_plane.get(i).copied().unwrap_or(0) }
    }
    fn world_cell(point:[i16;2])->i16 { ((point[1] as u16&0xfff0).wrapping_mul(8)).wrapping_add((point[0]>>4)as u16)as i16 }
    fn set_world_code(&mut self,cell:i16,code:u8) {
        if cell<0{return;}let i=cell as usize;let(x,y)=(i&127,i>>7);
        if let Some(c)=self.map.native_terrain_plane.get_mut(i){*c=code;}
        if x<self.map.width&&y<self.map.height { let j=y*self.map.width+x;if let Some(c)=self.map.terrain_codes.get_mut(j){*c=code&127;}
            if let Some(tile)=self.map.original_tiles.get_mut(j){*tile=(*tile&255)|(u16::from(code&128)<<1);}
            //36/46 mark OBJECT occupancy in the collision plane. The road or
            //ground artwork under that object keeps its original visual class.
            if !matches!(code&127,36|46){if let Some(t)=self.map.tiles.get_mut(j){*t=match code&127 {7=>1,19|43=>2,9|38|39|40|42=>3,23=>5,_=>0};}}
        }
    }
    fn world_saved(&self,point:[i16;2])->[u8;4] { let base=Self::world_cell(point);[0,1,128,129].map(|o|self.world_code(base.wrapping_add(o))) }
    fn world_restriction(&self,point:[i16;2])->u8 {
        let plane=(0..16384).map(|i|self.world_code(i as i16)).collect::<Vec<_>>();
        u8::from(super::native_ai_scoring::enclosed_object_area(&plane,point,|x,y|self.world_sector([x,y])))
    }
    fn make_object(&self,kind:u16,point:[i16;2],hp:u8)->Option<ObjectSlot> {
        let(sprite,flags,extent)=native::object_metadata(kind)?;
        Some(ObjectSlot { active:true,record:native::Object{position:point,flags13:native::initial_object_flags(kind,flags),metadata_flags18:flags,metadata_extent20:extent},hp,maximum:hp,kind,sprite,extent21:native::object_height(kind)?,bucket12:native_bucket(point),flags19:native::initial_object_scan_flags(kind),flags26:self.world_restriction(point),variant26:0,lifetime24:0,animation18:0,sprite_offset21:0,motion22:0,hardware:None,workshop_raw18_19:None,workshop_raw1b:None,flag_sector:None,flag_delay1f:0,saved:self.world_saved(point),unit:None,pickup:None,render:None,fire:None })
    }
    pub(crate) fn import_native_world(&mut self,v:&Value) {
        let previous=self.native_world.take();
        let old_saved=(0..200).map(|i|previous.as_ref().and_then(|s|s.objects[i].as_ref()).map(|o|o.saved).unwrap_or([0;4])).collect::<Vec<_>>();
        let mut plane=(0..16384).map(|i|self.world_code(i as i16)).collect::<Vec<_>>();
        //The visible A/B planes may already contain native wall autotiles.
        //Upgrade older packages whose full B plane accidentally lost bit7,
        //without rewriting any visual or compatibility cell during import.
        for y in 0..self.map.height.min(128){for x in 0..self.map.width.min(128){let cell=y*128+x;let visual=self.map.original_tiles.get(y*self.map.width+x).copied().unwrap_or(0);plane[cell]=(plane[cell]&127)|((visual&256)>>1)as u8;}}
        let initial_plane=plane.clone();self.map.native_terrain_plane=initial_plane.clone();
        let mut state=State::empty();state.order_object_pointer=previous.as_ref().map_or(0,|s|s.order_object_pointer);state.retired_preamble_offset21_24=previous.as_ref().and_then(|w|w.objects[24].as_ref().map(|o|o.sprite_offset21).or(w.retired_preamble_offset21_24));if let Some(old)=&previous{state.trail=old.trail.clone();state.trail.mission_reset();state.direct_palette=old.direct_palette;state.effect_quality=old.effect_quality;state.effects=old.effects.clone();for p in state.effects.iter_mut().flatten(){p.active=false;p.position[0]=-1000;}state.effect_cp_tints=vec![None;state.effects.len()];state.effect_tint=old.effect_tint;state.effect_owner=old.effect_owner;state.effect_maximum=old.effect_maximum;state.effect_origin=old.effect_origin;state.effect_flags=old.effect_flags;state.effect_fade=old.effect_fade;state.effect_sprite=old.effect_sprite;state.effect_suppression=old.effect_suppression;state.effect_world_mask4f=old.effect_world_mask4f;state.effect_wall_mask43=old.effect_wall_mask43;state.effect_world_suppression=old.effect_world_suppression;}state.seasonal_cps=if v.is_null(){previous.as_ref().and_then(|s|s.seasonal_cps.clone())}else{seasonal_game::CpPool::import(v)};state.fragments=building_fragments::State::import(previous.as_ref().map(|s|&s.fragments),v);state.fort_fragments=fort_fragments::State::import(previous.as_ref().map(|s|&s.fort_fragments),v);state.bridge_fx=previous.as_ref().map_or_else(Default::default,|s|s.bridge_fx);let mut units=BTreeSet::new();let mut pickups=BTreeSet::new();
        //460430 retires X only. Preserve raw inactive fields across missions;
        //Web render/unit bindings belong to the incoming mission instead.
        if let Some(old)=&previous{for(slot,object)in old.objects.iter().enumerate(){if let Some(mut o)=object.clone(){o.active=false;o.record.position[0]=-1;o.unit=None;o.pickup=None;o.render=None;o.fire=None;o.flag_sector=None;state.objects[slot]=Some(o);}}}
        //33ca0 calls41d280 before33700.17a50 already ran through the menu's
        //human-roster initialization, so authored stale CP low bits are not
        //the fort flags used here. Only the first/second native fort sectors
        //are excluded by41d280's runtime flags&3 gate.
        let mut forts=self.buildings.iter().filter(|b|b.kind=="fort").map(|b|(b.raw_base.map(|p|[p.0 as i16,p.1 as i16]).unwrap_or_else(||native_point(b.x,b.y)),b.sector)).collect::<Vec<_>>();forts.sort_by_key(|(p,_)|(p[1],p[0]));let fort_sectors=forts.iter().take(2).map(|(p,fallback)|self.world_sector(*p).unwrap_or(*fallback)).collect::<BTreeSet<_>>();
        let mut flag_sectors=v["sectors"].as_array().into_iter().flatten().filter(|s|s["nativeSlot"].as_u64().is_some_and(|n|n<64)&&s["rawFlagTile"].as_array().is_some_and(|p|p.len()==2)&&!s["countsForProduction"].as_bool().is_some_and(|b|!b)).collect::<Vec<_>>();flag_sectors.sort_by_key(|s|s["nativeSlot"].as_u64());
        for sector in flag_sectors{let id=number(sector,"id",-1);if id<0||fort_sectors.contains(&(id as usize)){continue;}let point=[(sector["rawFlagTile"][0].as_i64().unwrap_or(0)as i8 as i16).wrapping_shl(4),(sector["rawFlagTile"][1].as_i64().unwrap_or(0)as i8 as i16).wrapping_shl(4)];let cell=Self::world_cell(point);if cell>=0{if let Some(c)=plane.get_mut(cell as usize){*c=(*c&0xa3)|35;}}
            let Some(slot)=state.free()else{continue;};if let Some(mut flag)=self.make_object(28,point,50){flag.record.flags13=0;flag.flag_sector=Some(id as usize);flag.saved=old_saved[slot];flag.flags26=u8::from(super::native_ai_scoring::enclosed_object_area(&plane,point,|x,y|self.world_sector([x,y])));state.put_object(slot,flag);}
        }
        for (index,raw) in self.objects.iter().enumerate() {
            let kind=number(raw,"type",-1);if !(0..78).contains(&kind)||kind==37{continue;}
            let mut point=native_point(raw["x"].as_f64().unwrap_or(0.),raw["y"].as_f64().unwrap_or(0.));let bucket=native_bucket(point);
            if kind==7 { point=[point[0]&!15,(point[1]&!15).wrapping_sub(14)]; }
            let eq=number(raw,"equipmentId",255) as u16;
            let hp=number(raw,"nativeHp",equipment_kind(eq).map(|k|(stats(k).hp as i16/25)as i64).unwrap_or(0))as u8;
            let Some(mut object)=self.make_object(kind as u16,point,hp)else{continue;};
            object.hardware=u8::try_from(eq).ok().and_then(super::original_entry::equipment_attributes);if let Some(h)=object.hardware{object.maximum=h.byte(40);}if eq<=18||eq==255{object.workshop_raw1b=Some(object.hardware.map_or(0,|h|h.byte(27)));}
            object.record.flags13=number(raw,"nativeFlags13",i64::from(object.record.flags13))as u8;
            object.record.metadata_flags18=number(raw,"templateFlags",i64::from(object.record.metadata_flags18))as u8;
            object.record.metadata_extent20=number(raw,"templateExtent",i64::from(object.record.metadata_extent20))as i8;
            object.extent21=number(raw,"templateHeight",i64::from(object.extent21))as i8;
            if raw["nativeFlags13"].is_null(){object.record.flags13=native::initial_object_flags(kind as u16,object.record.metadata_flags18);}
            object.sprite=number(raw,"templateSprite",i64::from(object.sprite))as u16;object.render=Some(index);
            //43ef70 sets4d18cc=1 for authored APC31;43ecf0 selects30.
            //58af0 writes zero directly, so dynamic conversion keeps make_object's0.
            object.sprite_offset21=number(raw,"nativeSpriteOffset21",if kind==31{30}else{0})as i8;
            object.flags19=number(raw,"nativeFlags19",i64::from(object.flags19))as u8;
            object.bucket12=number(raw,"nativeBucket12",i64::from(bucket))as u8;
            object.flags26=number(raw,"nativeFlags26",i64::from(u8::from(super::native_ai_scoring::enclosed_object_area(&plane,point,|x,y|self.world_sector([x,y])))))as u8;
            if let Some(kind)=equipment_kind(eq) { object.unit=self.units.iter().find(|u|u.team<0&&u.kind==kind&&!units.contains(&u.id)&&native_point(u.x,u.y)==point).map(|u|u.id);if let Some(id)=object.unit{units.insert(id);} }
            if kind==10 { object.pickup=self.crates.iter().enumerate().find(|(i,c)|!pickups.contains(i)&&native_point(c.x,c.y)==point).map(|(i,_)|i);if let Some(id)=object.pickup{pickups.insert(id);} }
            let slot=raw["nativeSlot"].as_u64().map(|n|n as usize).filter(|n|*n<200&&state.objects[*n].as_ref().is_none_or(|o|!o.active)).or_else(||state.free());
            if let Some(slot)=slot{
                object.saved=old_saved[slot];
                let flags=object.record.metadata_flags18;
                if flags&4!=0||flags&3<=1{
                    let p=[point[0],point[1].wrapping_add(if flags&6==6{16}else{0})];let cell=Self::world_cell(p);
                    if cell>=0{if let Some(code)=plane.get_mut(cell as usize){object.saved[0]=*code;if !(71..=77).contains(&(kind as u16)){*code=if flags&4!=0{(*code&0xae)|46}else{(*code&0xa4)|36};}}}
                }
                state.put_object(slot,object);if let Some(value)=raw["nativeWreckVariant26"].as_i64(){state.objects[slot].as_mut().unwrap().variant26=value as i8;}
            }
        }
        if let Some(records)=v["nativeWorldBuildings"].as_array() {
            state.buildings=(0..35).map(|_|BuildingSlot{record:native::Building{position:[-1,-1],kind:-1,hp:-1},maximum:-1,binding:None,fort_proxy:false,repair:None,constructor:None,display:Default::default()}).collect();
            for b in records { let slot=number(b,"nativeSlot",-1);if !(0..35).contains(&slot){continue;}state.buildings[slot as usize]=BuildingSlot{record:native::Building{position:[number(b,"x",-1)as i16,number(b,"y",-1)as i16],kind:number(b,"kind",-1)as i8,hp:number(b,"hp",-1)as i16},maximum:number(b,"maxHp",-1)as i16,binding:b["buildingId"].as_u64().map(|n|n as usize),fort_proxy:b["fortProxy"].as_bool().unwrap_or(false),repair:None,constructor:None,display:super::original_building_display::Radar{word0a:number(b,"nativeWord0A",0)as i16,flags6:number(b,"flags6",0)as u8,notice15:number(b,"nativeByte15",0)as u8}}; }
        } else {
            // Synthetic arenas still use the native descriptors and HP rules.
            for b in &self.buildings { if let Some(kind)=building_kind(&b.kind){let mut position=native_point(b.x,b.y);if kind==2{position=b.raw_base.map(|p|[p.0 as i16,p.1 as i16+16]).unwrap_or([position[0],position[1]+16]);}state.buildings.push(BuildingSlot{record:native::Building{position,kind,hp:b.hp as i16},maximum:b.max_hp as i16,binding:Some(b.id),fort_proxy:kind==2,repair:None,constructor:None,display:Default::default()});} }
        }
        state.forts=self.buildings.iter().filter(|b|b.kind=="fort").map(|b|Fort{binding:b.id,base:b.raw_base.map(|p|[p.0 as i16,p.1 as i16]).unwrap_or_else(||native_point(b.x,b.y))}).collect();state.forts.sort_by_key(|f|(f.base[1],f.base[0]));state.forts.truncate(2);
        for b in v["bridges"].as_array().into_iter().flatten() { let id=number(b,"id",-1);if id<0{continue;}let hp=number(b,"hp",0)as i16;let flags=number(b,"nativeFlags15",if hp<=0{2}else{0})as u8;
            state.bridges.push(BridgeSlot{record:native::Bridge{position:native_point(b["x"].as_f64().unwrap_or(0.),b["y"].as_f64().unwrap_or(0.)),size:native_point(b["w"].as_f64().unwrap_or(0.),b["h"].as_f64().unwrap_or(0.)),hp:if hp<=0{-1}else{hp},maximum:number(b,"maxHp",600)as i16,flags15:flags},binding:id as usize,native_slot:number(b,"nativeSlot",number(b,"sourceOffset",id*26)/26)as usize,orientation:number(b,"nativeOrientation",0)as i16,retired_road:None,restored_terrain:Vec::new(),raw_tail:[0;32],native_work_owned:false}); }
        state.bridges.sort_by_key(|b|b.native_slot);
        state.load_native_bridge_records(previous.as_ref(),v);
        //Only actual native OBJECT writes change the planes. Synthetic fixtures
        //can intentionally have compatibility collision classes without codes.
        for (cell,code) in plane.into_iter().enumerate(){if code!=initial_plane[cell]{self.set_world_code(cell as i16,code);}}
        //4097b0 initializes already-destroyed bridges before live groups spawn.
        for bridge in &mut state.bridges{if bridge.record.position[0]!=-1&&bridge.record.flags15&2!=0{self.collapse_native_bridge_terrain(bridge);for(cell,&owner)in self.map.bridge_base.iter().enumerate(){if owner==bridge.binding as i32{if let Some(tile)=self.map.tiles.get_mut(cell){*tile=3;}}}}}
        self.native_world=Some(state);self.import_native_trail_buildings(v);self.import_native_crane_data(v);let forts=self.buildings.iter().filter(|b|b.kind=="fort").map(|b|b.id).collect::<Vec<_>>();for id in forts{self.copy_native_fort_producer_health(id,true);}self.sync_native_world();self.initialize_native_fort_fragments();
    }
    pub(crate) fn sync_native_world(&mut self) {
        let Some(mut state)=self.native_world.take()else{if self.map.original{self.import_native_world(&Value::Null);}return;};
        for object in state.objects.iter_mut().flatten().filter(|o|o.active) {
            if let Some(id)=object.unit { if let Some(u)=self.units.iter().find(|u|u.id==id&&u.team<0&&u.crew.is_none()&&original_combat::alive(true,u.hp)){object.record.position=native_point(u.x,u.y);object.bucket12=native_bucket(object.record.position);object.hp=(u.hp as i16/25)as u8;}else{object.active=false;} }
            if object.pickup.is_some_and(|id|self.crates.get(id).is_none_or(|c|c.taken)){object.active=false;}
            if let Some(id)=object.fire { if let Some(f)=self.fires.iter().find(|f|f.id==id){object.record.position=native_point(f.x,f.y);object.bucket12=native_bucket(object.record.position);} }
        }
        // Retire old renderer bindings before a free slot can be reused by a
        // different entity. Keeping just the new slot would lose this link.
        for object in state.objects.iter().flatten().filter(|o|!o.active){if let Some(index)=object.render{if let Some(v)=self.objects.get_mut(index){v["nativeActive"]=json!(false);}}}
        for u in self.units.iter().filter(|u|u.team<0&&u.crew.is_none()&&original_combat::alive(true,u.hp)) {
            if state.objects.iter().flatten().any(|o|o.active&&o.unit==Some(u.id)){continue;}let Some(kind)=object_kind(&u.kind)else{continue;};let Some(slot)=state.free()else{break;};
            if let Some(mut object)=self.make_object(kind,native_point(u.x,u.y),(u.hp as i16/25)as u8){object.unit=Some(u.id);object.hardware=(0..19u8).find(|&eq|equipment_kind(u16::from(eq))==Some(u.kind.as_str())).and_then(super::original_entry::equipment_attributes);if let Some(h)=object.hardware{object.maximum=h.byte(40);}object.saved=state.objects[slot].as_ref().map(|o|o.saved).unwrap_or([0;4]);state.put_object(slot,object);}
        }
        for (id,c) in self.crates.iter().enumerate().filter(|(_,c)|!c.taken){if state.objects.iter().flatten().any(|o|o.active&&o.pickup==Some(id)){continue;}let Some(slot)=state.free()else{break;};if let Some(mut object)=self.make_object(10,native_point(c.x,c.y),0){object.pickup=Some(id);state.put_object(slot,object);}}
        for record in &mut state.buildings { if !record.fort_proxy {if let Some(b)=record.binding.and_then(|id|self.buildings.iter().find(|b|b.id==id)){record.record.hp=b.hp as i16;record.record.position=native_point(b.x,b.y);}} }
        for record in &mut state.bridges {if record.native_work_owned{continue;}if let Some(&hp)=self.map.bridge_hp.get(record.binding){let hp=hp as i16;
            if record.record.flags15&2!=0 {if hp>=record.record.maximum {
                record.record.hp=hp;record.record.flags15&=!(2|32);
                for &(cell,code)in &record.restored_terrain{self.set_world_code(cell,code);}
                if let Some((slot,bounds))=record.retired_road.take(){if let Some(region)=self.map.native_road_graph.as_mut().and_then(|g|g.regions.get_mut(slot)){region.bounds=bounds;}}
                record.restored_terrain.clear();
            }}else if hp>0{record.record.hp=hp;if hp>record.record.maximum>>1{record.record.flags15&=!32;}}
        }}
        for (slot,o) in state.objects.iter().enumerate().filter_map(|(slot,o)|o.as_ref().map(|o|(slot,o))){if let Some(i)=o.render{if let Some(v)=self.objects.get_mut(i){v["nativeActive"]=json!(o.active);v["nativeSlot"]=json!(slot);v["nativeHp"]=json!(o.hp);v["nativeFlags13"]=json!(o.record.flags13);v["nativeFlags19"]=json!(o.flags19);v["nativeFlags26"]=json!(o.flags26);v["nativeSpriteOffset21"]=json!(o.sprite_offset21);v["nativeAnimation18"]=json!(o.animation18);if o.active{v["x"]=json!(i32::from(o.record.position[0])*2);v["y"]=json!(i32::from(o.record.position[1])*2);}}}}
        self.native_world=Some(state);
    }
    pub(crate) fn native_work_health(&self,b:&Building)->(f64,f64) {
        self.native_world.as_ref().and_then(|s|s.buildings.iter().find(|r|r.fort_proxy&&r.binding==Some(b.id))).map(|r|(if r.record.kind==-1{0.}else{f64::from(r.record.hp)},f64::from(r.maximum))).unwrap_or((b.hp,b.max_hp))
    }
    ///40b250 initializes unconditionally.40b2a0's regular callback first
    ///rejects a zero-HP record, then calls40b1c0 before40a6d0.
    pub(crate) fn sync_native_fort_producer_health(&mut self,id:usize){self.copy_native_fort_producer_health(id,false);}
    fn copy_native_fort_producer_health(&mut self,id:usize,initial:bool){let Some(b)=self.buildings.iter().find(|b|b.id==id&&b.kind=="fort")else{return;};let(hp,max)=(b.hp as i16,b.max_hp as i16);if let Some(state)=&mut self.native_world{for record in state.buildings.iter_mut().filter(|r|r.fort_proxy&&r.record.kind==2&&r.binding==Some(id)&&(initial||r.record.hp!=0)){record.record.hp=hp;record.maximum=max;}}}
    pub(crate) fn native_producer_position(&self,b:&Building)->[i16;2]{self.native_world.as_ref().and_then(|s|s.buildings.iter().find(|r|r.binding==Some(b.id))).map(|r|r.record.position).unwrap_or_else(||if b.kind=="fort"{b.raw_base.map(|p|[p.0 as i16,(p.1 as i16).wrapping_add(16)]).unwrap_or_else(||{let mut p=native_point(b.x,b.y);p[1]=p[1].wrapping_add(16);p})}else{native_point(b.x,b.y)})}
    pub(crate) fn native_ai_world_objects(&self)->Option<Vec<task::WorldObject>> {Some(self.native_world.as_ref()?.objects.iter().enumerate().filter_map(|(slot,o)|{let o=o.as_ref()?;o.active.then(||task::WorldObject{id:slot,position:o.record.position,definition:Some(o.kind as i16),flags13:o.record.flags13,flags26:o.flags26,sector:self.world_sector(o.record.position)})}).collect())}
    pub(crate) fn native_neutral_unit_sprite(&self,id:usize)->Option<i16>{let o=self.native_world.as_ref()?.objects.iter().flatten().find(|o|o.active&&o.unit==Some(id))?;Some((o.sprite as i16).wrapping_add(i16::from(o.animation18 as i8)).wrapping_add(i16::from(o.sprite_offset21)))}
    pub(crate) fn native_object_visuals(&self)->Vec<Value>{self.native_world.as_ref().map(|s|s.objects.iter().flatten().filter_map(|o|o.render.map(|id|json!({"id":id,"nativeActive":o.active,"x":i32::from(o.record.position[0])*2,"y":i32::from(o.record.position[1])*2,"nativeAnimation18":o.animation18,"nativeSpriteOffset21":o.sprite_offset21}))).collect()).unwrap_or_default()}
    pub(crate) fn native_object_unit(&self,slot:usize)->Option<usize>{self.native_world.as_ref()?.objects.get(slot)?.as_ref().filter(|o|o.active)?.unit}
    ///460430 initializes only X toÃ¢Ë†â€™1. A never-used cold BSS slot has Y0;
    ///retired records retain their actual old Y for43bdd0's raw read.
    pub(crate) fn native_object_raw_position(&self,slot:u8)->Option<[i16;2]>{Some(self.native_world.as_ref()?.objects.get(usize::from(slot))?.as_ref().map_or([-1,0],|o|o.record.position))}
    pub(crate) fn native_object_hardware_input(&self,slot:u8)->Option<super::original_entry::ObjectHardware>{let o=self.native_world.as_ref()?.objects.get(usize::from(slot))?.as_ref().filter(|o|o.active)?;let mut h=o.hardware?;h.set_byte(15,o.hp);h.set_byte(40,o.maximum);Some(h)}
    pub(crate) fn native_object_slot(&self,unit:usize)->Option<u8>{self.native_world.as_ref()?.objects.iter().position(|o|o.as_ref().is_some_and(|o|o.active&&o.unit==Some(unit))).map(|n|n as u8)}
    ///Ordinary41c600 query (6a=6b=0); blast-specific overrides remain explicit
    ///arguments at their callers and do not leak into a later UNIT callback.
    pub(crate) fn native_first_object_contact(&self,point:[i16;2])->Option<u8>{let state=self.native_world.as_ref()?;let objects=state.objects.iter().map(|o|o.as_ref().filter(|o|o.active).map(|o|o.record).unwrap_or(native::Object{position:[-1,-1],flags13:0,metadata_flags18:0,metadata_extent20:0})).collect::<Vec<_>>();native::first_object(&objects,point,false,false).map(|slot|slot as u8)}
    ///45f070's B3 projection after classification. Missing raw world input is
    ///unavailable, not a fabricated43ea80 miss.8/9 preserve the old marker.
    pub(crate) fn native_order_object_marker(&mut self,event:super::original_orders_game::Event)->Option<i8>{
        if matches!(event.opcode,8|9){return None;}
        if !matches!(event.dispatched_opcode,2|5){return Some(-1);}
        let state=self.native_world.as_mut()?;
        let objects=state.objects.iter().map(|o|o.as_ref().map(|o|o.record).unwrap_or(native::Object{position:[-1,0],flags13:0,metadata_flags18:0,metadata_extent20:0})).collect::<Vec<_>>();
        let point=if event.dispatched_opcode==5{[event.waypoint[1],event.metadata]}else{event.waypoint};
        let hit=native::order_object(&objects,point);
        if let Some(slot)=hit{state.order_object_pointer=0x612f80+slot as u32*44;}
        Some(if hit.is_some(){1}else{-1})
    }
    pub(crate) fn native_world_scan_objects(&self)->Vec<ScanObject>{self.native_world.as_ref().map(|s|s.objects.iter().enumerate().filter_map(|(slot,o)|o.as_ref().filter(|o|o.active).map(|o|ScanObject{slot:slot as u8,position:o.record.position,bucket12:o.bucket12,flags13:o.record.flags13,flags19:o.flags19,flags26:o.flags26,definition:o.kind,sprite:o.sprite,template_flags18:o.record.metadata_flags18,extent20:o.record.metadata_extent20,extent21:o.extent21,unit:o.unit,flag_sector:o.flag_sector,flag_delay1f:o.flag_delay1f})).collect()).unwrap_or_default()}
    ///Readonly43f260 projection, in the actual200-slot insertion order.
    ///Template19 is depth; ObjectSlot.flags19 is a different runtime byte.
    pub(crate) fn native_display_objects(&self)->Vec<super::original_display::ObjectBinding>{use super::original_display as display;self.native_world.as_ref().map(|s|s.objects.iter().enumerate().filter_map(|(slot,o)|{let o=o.as_ref().filter(|o|o.active&&o.record.position[0]>0)?;let depth=*display::OBJECT_DEPTH.get(usize::from(o.kind))?;Some(display::ObjectBinding{slot:slot as u8,position:o.record.position,record:display::object(display::ObjectInput{position:o.record.position,base:o.sprite as i16,animation:o.animation18 as i8,sprite_offset:o.sprite_offset21,flags:o.record.flags13,depth}),unit:o.unit,object:o.render,flag:o.flag_sector,pickup:o.pickup,fire:o.fire})}).collect()).unwrap_or_default()}
    ///43f260 submits after this slot's lifetime admission, never from a
    ///snapshot. Later callbacks cannot rewrite these rows or repeat CRT draws.
    fn capture_native_object_display(&mut self,slot:usize){
        use super::original_display as display;
        let Some(o)=self.native_world.as_ref().and_then(|s|s.objects.get(slot)).and_then(Option::as_ref).filter(|o|o.active&&o.record.position[0]>0).cloned()else{return;};let Some(&depth)=display::OBJECT_DEPTH.get(usize::from(o.kind))else{return;};let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));
        let input=display::ObjectInput{position:o.record.position,base:o.sprite as i16,animation:o.animation18 as i8,sprite_offset:o.sprite_offset21,flags:o.record.flags13,depth};if self.capture_native_seasonal_object(slot,input,camera,size){return;}let draw=match display::objects::project(input,o.variant26,i16::from(self.native_flag_frame),self.native_object_frame,camera,size){Ok(draw)=>draw,Err(value)=>{self.native_route_boundary=Some(super::original_path_contact::game::Boundary::ObjectOverlayVariant(slot,value));return;}};
        for(part,r)in draw.records.into_iter().enumerate(){self.store_native_object_picture(slot,r,if part==0{"object"}else{"wreckOverlay"},part!=0);}
        if let Some(point)=draw.spark{let probe=if self.multiplayer{0}else{self.native_cosmetic_random()};if display::objects::spark_probe(probe){let random=if self.multiplayer{0}else{self.native_cosmetic_random()};self.native_ambient_animations.spawn_spark(point,random);}}
    }
    fn store_native_object_picture(&mut self,slot:usize,r:super::original_display::Record,source:&str,literal:bool){if let Some(rows)=&mut self.native_display.objects{let mut row=json!({"kind":"sprite","source":source,"id":slot,"nativeSlot":slot,"queueY":i32::from(r.y)*2,"palette":r.palette,"sprite":r.sprite,"x":i32::from(r.x)*2,"y":i32::from(r.y.wrapping_add(i16::from(r.offset)))*2,"scale":1.,"rotation":0.});if literal{row["nativeLiteralPalette"]=json!(true);}rows.push((r,row));}}
    pub(crate) fn native_display_building_order(&self)->Vec<usize>{self.native_world.as_ref().map(|s|s.buildings.iter().filter(|b|b.record.kind!=-1).filter_map(|b|b.binding).collect()).unwrap_or_default()}
    pub(crate) fn native_radar_building_ids(&self)->Vec<usize>{self.native_world.as_ref().map(|s|s.buildings.iter().filter(|b|b.record.kind==8).filter_map(|b|b.binding).collect()).unwrap_or_default()}
    fn native_radar_inputs(&self)->Vec<(usize,Option<usize>,super::original_building_display::Radar,super::original_building_display::Input)>{let Some(world)=&self.native_world else{return vec![];};let(camera,size)=self.native_view.unwrap_or(([0,0],[(self.map.width*16)as i16,(self.map.height*16)as i16]));world.buildings.iter().enumerate().filter(|(_,b)|b.record.kind==8).map(|(slot,b)|{let owner=b.binding.and_then(|id|self.buildings.iter().find(|v|v.id==id)).map(|b|b.team).or_else(||self.world_sector(b.record.position).and_then(|id|self.sectors.iter().find(|s|s.id==id)).map(|s|s.owner)).unwrap_or(-1);(slot,b.binding,b.display,super::original_building_display::Input{position:b.record.position,maximum:b.maximum,hp:b.record.hp,owner:if owner<0{4}else{owner as i8},local_team:self.native_feedback.local_team as i8,pulse:self.native_callback_mode!=0,camera,size})}).collect()}
    pub(crate) fn native_radar_display_projection(&self)->Vec<(usize,Option<usize>,super::original_building_display::Result)>{self.native_radar_inputs().into_iter().map(|(slot,id,mut state,input)|(slot,id,state.advance(input))).collect()}
    pub(crate) fn advance_native_radar_display(&mut self)->Vec<(usize,Option<usize>,super::original_building_display::Result)>{let inputs=self.native_radar_inputs();let mut output=Vec::new();for(slot,id,mut state,input)in inputs{let out=state.advance(input);if let Some(b)=self.native_world.as_mut().and_then(|w|w.buildings.get_mut(slot)){b.display=state;if let Some(c)=&mut b.constructor{c.flags6=state.flags6;}}if let Some(id)=id{self.native_factory_flags6.insert(id,state.flags6);}output.push((slot,id,out));}output}
    pub(crate) fn advance_native_radar_display_for(&mut self,id:usize)->Vec<(usize,Option<usize>,super::original_building_display::Result)>{let mut output=Vec::new();for(slot,binding,mut state,input)in self.native_radar_inputs().into_iter().filter(|(_,binding,_,_)|*binding==Some(id)){let before=state;let out=state.advance(input);self.emit_native_radar_audio(slot,before,input,out,state.word0a);if let Some(b)=self.native_world.as_mut().and_then(|w|w.buildings.get_mut(slot)){b.display=state;if let Some(c)=&mut b.constructor{c.flags6=state.flags6;}}self.native_factory_flags6.insert(id,state.flags6);output.push((slot,binding,out));}output}
    pub(crate) fn native_radar_raw_state(&self,slot:usize)->Option<super::original_building_display::Radar>{Some(self.native_world.as_ref()?.buildings.get(slot)?.display)}
    pub(crate) fn native_world_buildings(&self)->Option<Vec<native::Building>>{Some(self.native_world.as_ref()?.buildings.iter().map(|b|b.record).collect())}
    ///The constructor delta records only source-written fields; None bytes
    ///retain their imported value and are not fabricated from cold defaults.
    pub(crate) fn set_native_building_constructor(&mut self,slot:usize,fields:super::original_factory::ConstructorFields){let repair=if let Some(b)=self.native_world.as_mut().and_then(|s|s.buildings.get_mut(slot)){if let Some(maximum)=fields.maximum1c{b.maximum=maximum;}b.display.flags6=fields.flags6;if b.record.kind==8{b.display.construct();}if b.record.kind==0{let mut r=b.repair.unwrap_or_default();r.set_word(0,b.record.position[0]);r.set_word(2,b.record.position[1]);r.set_word(30,b.record.hp);r.construct(fields.flags6,b.maximum,fields.field11);b.repair=Some(r);}b.constructor=Some(fields);b.record.kind==0}else{false};self.initialize_native_building_fragments(slot);self.initialize_native_trail_build_constructor(slot,fields);if repair{self.initialize_native_workshop_terrain(slot);}}

    pub(crate) fn native_building_constructor(&self,slot:usize)->Option<super::original_factory::ConstructorFields>{self.native_world.as_ref()?.buildings.get(slot)?.constructor}
    pub(crate) fn native_hardware_building_owner_at(&self,slot:Option<usize>)->Option<u8>{let b=self.native_world.as_ref()?.buildings.get(slot?)?;let b=self.buildings.iter().find(|v|Some(v.id)==b.binding)?;u8::try_from(b.team).ok()}
    ///41c3f0 reads the actual BUILD24 work and signed3a door offset.
    pub(crate) fn native_hardware_repair_at(&self,point:[i16;2])->Result<Option<([i16;2],u8)>,super::original_path_contact::game::Boundary>{
        let Some(world)=self.native_world.as_ref()else{return Err(super::original_path_contact::game::Boundary::MissingHardwareProfile);};
        for(slot,b)in world.buildings.iter().enumerate().take(35).filter(|(_,b)|b.record.kind==0&&b.record.hp!=0){
            let Some(raw)=b.repair else{if b.record.position[0]<=point[0]&&point[0]<=b.record.position[0].wrapping_add(80)&&(i32::from(point[1])-i32::from(b.record.position[1])).abs()<=159{return Err(super::original_path_contact::game::Boundary::HardwareRepairState(slot));}continue;};
            //A compatibility job has no source BUILD24 yet; never pretend its
            //bay is free. Authored native admissions use the raw slot below.
            if b.binding.is_some_and(|id|self.units.iter().any(|u|u.workshop.as_ref().is_some_and(|w|w.building_id==id&&w.native_slot.is_none()&&w.phase!="approach"))){return Err(super::original_path_contact::game::Boundary::HardwareRepairState(slot));}
            let input=super::original_attack_click::RepairBuilding{position:b.record.position,kind:b.record.kind,hp:b.record.hp,remaining:raw.remaining(),offset3a:raw.bytes[58]as i8,owner:self.native_workshop_owner(slot).unwrap_or(4)as u8};
            if let Some((_,at,owner))=super::original_attack_click::repair_at(&[input],point){return Ok(Some((at,owner as u8)));}
        }Ok(None)
    }

    pub(crate) fn native_world_bridges(&self)->Option<Vec<native::Bridge>>{let state=self.native_world.as_ref()?;Some((0..12).map(|slot|state.bridges.iter().find(|b|b.native_slot==slot).map(|b|b.record).unwrap_or(native::Bridge{position:[-1,-1],size:[0,0],hp:0,maximum:0,flags15:2})).collect())}
    pub(crate) fn native_fort_bases(&self)->Option<[[i16;2];2]>{let s=self.native_world.as_ref()?;Some([s.forts.first()?.base,s.forts.get(1)?.base])}
    pub(crate) fn native_ai_world_buildings(&self,team:i32)->Option<Vec<task::BuildingRecord>>{let s=self.native_world.as_ref()?;Some(s.buildings.iter().enumerate().map(|(slot,b)|{let binding=b.binding.and_then(|id|self.buildings.iter().find(|v|v.id==id));let shape=usize::try_from(b.record.kind).ok().and_then(|kind|native::BUILDING_SHAPES.get(kind));task::BuildingRecord{id:b.binding.unwrap_or(65536+slot),position:b.record.position,kind:b.record.kind,owner:binding.map_or(4,|v|if v.team==team{1}else if v.team>=0{0}else{4}),sector:binding.map(|v|v.sector).or_else(||self.world_sector(b.record.position)),hit_points:b.record.hp,attack_offset:shape.map_or([0,0],|v|[v[2],v[3]])}}).collect())}
    pub(crate) fn set_native_flag_delay(&mut self,slot:u8,delay:u8){if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(usize::from(slot))).and_then(Option::as_mut){o.flag_delay1f=delay;}}
    ///43f260, after UNIT updates and before the projectile callbacks. The
    ///byte is tested as signed; values128..255 do not count toward zero.
    pub(crate) fn update_native_world_object_delays(&mut self){self.update_native_world_objects();}
    ///43f260 processes each live OBJECT slot completely before the next one:
    ///delay, fire/door animation, then lifetime. Contacts may allocate later slots.
    pub(crate) fn update_native_world_objects(&mut self){
        self.sync_native_world();
        self.native_display.objects=Some(Vec::new());
        for slot in 0..200{
            let base=self.native_world.as_mut().and_then(|s|s.objects.get_mut(slot)).and_then(Option::as_mut).filter(|o|o.active&&o.record.position[0]>0).map(|o|{if (o.flag_delay1f as i8)>0{o.flag_delay1f-=1;}o.sprite});
            let Some(base)=base else{continue;};if base==5215{continue;}
            if self.motion_tick&1==0{
                if [557,469,5836,5840,5844,5848,5854].contains(&base){self.advance_native_world_fire(slot);}
                else if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(slot)).and_then(Option::as_mut){o.animation18=o.animation18.wrapping_add(o.motion22 as u8);if [0,4].contains(&o.animation18){o.motion22=0;}}
            }
            let expired=self.native_world.as_mut().and_then(|s|s.objects.get_mut(slot)).and_then(Option::as_mut).filter(|o|o.active).and_then(|o|super::original_death::advance_object_lifetime(&mut o.lifetime24).then_some(o.record.position));
            if let Some(point)=expired{let origin=point.map(|c|c.wrapping_add(1));let selected=self.native_world.as_ref().and_then(|s|{let objects=s.objects.iter().map(|o|o.as_ref().filter(|o|o.active).map(|o|o.record).unwrap_or(native::Object{position:[-1,-1],flags13:0,metadata_flags18:0,metadata_extent20:0})).collect::<Vec<_>>();native::blast_object(&objects,origin,false,false)});if let Some((hit,sample))=selected{let d=[[4,16],[20,16],[20,32],[4,32]][sample];self.damage_native_world_object_mode(hit,80,[origin[0].wrapping_add(d[0]),origin[1].wrapping_add(d[1])],true);}if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(slot)).and_then(Option::as_mut){o.active=false;o.record.position[0]=-1;if let Some(raw)=o.render.and_then(|i|self.objects.get_mut(i)){raw["nativeActive"]=json!(false);}}}
            self.capture_native_object_display(slot);
            if let Some(o)=self.native_world.as_ref().and_then(|s|s.objects[slot].as_ref()){if let Some(raw)=o.render.and_then(|i|self.objects.get_mut(i)){raw["nativeAnimation18"]=json!(o.animation18);raw["nativeSpriteOffset21"]=json!(o.sprite_offset21);}}
        }
    }
    pub(crate) fn configure_native_world_fire_slot(&mut self,slot:usize,point:[i16;2],base:u16,frame:u8,lifetime:i8){if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(slot)).and_then(Option::as_mut){o.record.position=point;o.bucket12=native_bucket(point);o.sprite=base;o.animation18=frame;o.motion22=lifetime;if let Some(kind)=(71..=77).find(|&k|native::object_metadata(k).is_some_and(|r|r.0==base)){o.kind=kind;let(_,flags,extent)=native::object_metadata(kind).unwrap();o.record.metadata_flags18=flags;o.record.metadata_extent20=extent;o.extent21=native::object_height(kind).unwrap_or(0);}}}
    fn advance_native_world_fire(&mut self,slot:usize){
        let Some(snapshot)=self.native_world.as_ref().and_then(|s|s.objects.get(slot)).and_then(Option::as_ref).cloned()else{return;};let mut frame=snapshot.animation18;let pulse=fire::advance_frame(snapshot.sprite,&mut frame);if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects[slot].as_mut()){o.animation18=frame;}
        if pulse{let point=snapshot.record.position;if self.native_random()&7==7{self.spawn_native_smoke(point);}let damage=self.native_fire_strength(i32::from(point[0]),i32::from(point[1]),self.native_fire_base_strength);if let Some(s)=&mut self.native_world{s.effect_world_mask4f=255;s.effect_wall_mask43=1;s.effect_world_suppression=1;}self.native_blast_environment(i32::from(point[0]),i32::from(point[1]),damage,255,true);if let Some(s)=&mut self.native_world{s.effect_wall_mask43=0;s.effect_world_suppression=0;}self.native_direct_environment_contact_mode([point[0],point[1],point[0].wrapping_add(24),point[1].wrapping_add(24)],damage,true);if let Some(s)=&mut self.native_world{s.effect_world_mask4f=0;}let retired=self.native_world.as_mut().and_then(|s|s.objects[slot].as_mut()).is_some_and(|o|fire::finish_pulse(&mut o.motion22));if retired{self.spawn_native_smoke(point);if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects[slot].as_mut()){o.active=false;o.record.position[0]=-1;}}}
        if let Some(o)=self.native_world.as_ref().and_then(|s|s.objects[slot].as_ref()){if let Some(id)=o.fire{if !o.active{self.fires.retain(|f|f.id!=id);}else if let Some(f)=self.fires.iter_mut().find(|f|f.id==id){f.sync_native_object(o.animation18,o.motion22);}}if let Some(raw)=o.render.and_then(|i|self.objects.get_mut(i)){raw["nativeActive"]=json!(o.active);}}
    }
    pub(crate) fn set_native_object_scan_mask(&mut self,slot:u8,mask:u8){if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(usize::from(slot))).and_then(Option::as_mut){o.flags19=mask;}}
    pub(crate) fn apply_native_ambient_object(&mut self,slot:u8,position:[i16;2],flags13:u8){if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(usize::from(slot))).and_then(Option::as_mut){o.record.position=position;o.record.flags13=flags13;o.active=position[0]!=-1;if let Some(v)=o.render.and_then(|i|self.objects.get_mut(i)){v["nativeActive"]=json!(o.active);v["nativeFlags13"]=json!(flags13);v["x"]=json!(i32::from(position[0])*2);v["y"]=json!(i32::from(position[1])*2);}}}
    ///Reservation is an explicit native contact transition, never inferred from
    ///the browser order string.4bba0 releases bit0 on a detached member's slot.
    pub(crate) fn reserve_native_object(&mut self,slot:u8)->bool{let Some(object)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(usize::from(slot))).and_then(Option::as_mut).filter(|o|o.active)else{return false;};object.record.flags13|=1;true}
    pub(crate) fn release_native_object(&mut self,slot:u8){if let Some(object)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(usize::from(slot))).and_then(Option::as_mut){object.record.flags13&=!1;}}
    pub(crate) fn set_native_object_animation(&mut self,slot:u8,frame:u8,motion:i8){if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(usize::from(slot))).and_then(Option::as_mut){o.animation18=frame;o.motion22=motion;}}
    pub(crate) fn native_object_animation(&self,slot:u8)->Option<(u8,i8)>{self.native_world.as_ref()?.objects.get(usize::from(slot))?.as_ref().filter(|o|o.active).map(|o|(o.animation18,o.motion22))}
    pub(crate) fn set_native_object_sprite_offset(&mut self,slot:u8,offset:i8){if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(usize::from(slot))).and_then(Option::as_mut){o.sprite_offset21=offset;if let Some(raw)=o.render.and_then(|i|self.objects.get_mut(i)){raw["nativeSpriteOffset21"]=json!(offset);}}}
    pub(crate) fn native_object_sprite_offset(&self,slot:u8)->Option<i8>{self.native_world.as_ref()?.objects.get(usize::from(slot))?.as_ref().filter(|o|o.active).map(|o|o.sprite_offset21)}
    ///59d80's successful APC portal cancellation. The distinct flags26 field
    ///does not participate:612f92 is OBJECT+12h and612fa2 is OBJECT+22h.
    pub(crate) fn clear_native_apc_object_contact(&mut self,slot:u8)->bool{let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(usize::from(slot))).and_then(Option::as_mut).filter(|o|o.active&&o.kind==31&&o.sprite==5173&&o.animation18!=0)else{return false;};o.motion22=-1;true}
    pub(crate) fn clear_native_hardware_restrictions(&mut self,sector:usize){let Some(mut objects)=self.native_ai_world_objects()else{return;};task::clear_hardware_capture_restrictions(&mut objects,sector,self.map.native_level_record);if let Some(state)=&mut self.native_world{for o in objects{if let Some(Some(slot))=state.objects.get_mut(o.id){slot.flags26=o.flags26;}}}}
    pub(crate) fn reserve_native_world_fire_slot(&mut self)->Option<usize>{self.sync_native_world();let mut state=self.native_world.take()?;let slot=state.free();if let Some(slot)=slot{if let Some(mut object)=self.make_object(71,[0,0],0){object.fire=Some(slot);object.flags26=0;state.put_object(slot,object);}}self.native_world=Some(state);slot}
    pub(crate) fn release_native_world_fire_slot(&mut self,id:usize){if let Some(state)=&mut self.native_world{if let Some(Some(object))=state.objects.get_mut(id){if object.fire==Some(id){object.active=false;}}}}
    pub(crate) fn native_fire_strength(&self,x:i32,y:i32,base:u8)->u8 {
        let point=[x as i16,y as i16];let hit=self.native_world.as_ref().is_some_and(|s|native::first_building(&s.buildings.iter().map(|b|b.record).collect::<Vec<_>>(),point,true,false).is_some()||s.bridges.iter().take(12).any(|b|native::bridge_contains(b.record,point)));
        native::fire_strength(base,hit)
    }
    pub(crate) fn native_blast_environment(&mut self,x:i32,y:i32,strength:u8,team:i32,fire_mode:bool){self.native_blast_environment_mode(x,y,strength,team,fire_mode,false);}
    ///420200's scene queries precede its spray and terrain dispatcher. The
    ///Owner4 caller supplies that dispatcher at the correct later stage.
    pub(super)fn native_building_part_environment(&mut self,p:[i16;2]){self.native_blast_environment_mode(i32::from(p[0]),i32::from(p[1]),50,255,false,true);}
    fn native_blast_environment_mode(&mut self,x:i32,y:i32,strength:u8,team:i32,fire_mode:bool,defer_terrain:bool){
        self.sync_native_world();let origin=[x as i16,y as i16];
        let selected=self.native_world.as_ref().and_then(|s|{if s.suppress_objects!=0{return None;}let objects=s.objects.iter().map(|o|o.as_ref().filter(|o|o.active).map(|o|o.record).unwrap_or(native::Object{position:[-1,-1],flags13:0,metadata_flags18:0,metadata_extent20:0})).collect::<Vec<_>>();native::blast_object(&objects,origin,false,false)});
        if let Some((slot,sample))=selected {let delta=[[4,16],[20,16],[20,32],[4,32]][sample];self.damage_native_world_object(slot,strength,[origin[0].wrapping_add(delta[0]),origin[1].wrapping_add(delta[1])]);}
        let Some(mut state)=self.native_world.take()else{return;};
        let mut collapse=None;
        let point=native::building_blast_point(origin,fire_mode);let records=state.buildings.iter().map(|b|b.record).collect::<Vec<_>>();
        if let Some((slot,_))=native::first_building(&records,point,true,false){let b=&mut state.buildings[slot];let damage=native::building_damage(b.record.hp,i16::from(strength));b.record.hp=damage.hp;if state.trail.known_builds[slot]{trail::raw_set(&mut state.trail.builds[slot],30,damage.hp);}
            if !b.fort_proxy {if let Some(building)=b.binding.and_then(|id|self.buildings.iter_mut().find(|b|b.id==id)){building.hp=f64::from(damage.hp);if damage.destroyed{building.ready_guns.clear();self.events.push(json!({"kind":"buildingLost","team":building.team,"id":building.id}));}}}else if damage.destroyed{b.record.kind=-1;}
            if damage.destroyed&&!b.fort_proxy{if let Some(shape)=usize::try_from(b.record.kind).ok().and_then(|k|native::BUILDING_SHAPES.get(k)){
                let [x,y]=b.record.position;collapse=Some([x.wrapping_add(16),y.wrapping_add(16),x.wrapping_add(i16::from(shape[0])*16).wrapping_sub(16),y.wrapping_add(i16::from(shape[1])*16).wrapping_sub(16)]);
            }
                //40e790 pointer stage: after the separate owner4/machinery
                //prefix, before the existing occupant/region collapse.
                self.stamp_native_building_ruin(&mut state,slot);self.native_building_particle_prefix(&mut state,slot);
                Self::retire_native_building_fragment_prefix(&mut state,slot);
            }
        }
        self.native_world=Some(state);if let Some(rect)=collapse{self.collapse_native_units(rect,false);}let Some(state)=self.native_world.take()else{return;};
        let bases=std::array::from_fn(|i|state.forts.get(i).map(|f|f.base).unwrap_or([32767,32767]));let hp=std::array::from_fn(|i|state.forts.get(i).and_then(|f|self.buildings.iter().find(|b|b.id==f.binding)).map(|b|b.hp as i16).unwrap_or(0));
        let damage=native::fort_damage(origin,if (0..4).contains(&team){Some(team as u8)}else{None},fire_mode,bases,hp,i16::from(strength));
        for i in 0..state.forts.len(){if let Some(b)=self.buildings.iter_mut().find(|b|b.id==state.forts[i].binding){b.hp=f64::from(damage.hp[i]);if damage.destroyed[i]{self.winner=Some(1-b.team);self.events.push(json!({"kind":"buildingLost","team":b.team,"id":b.id}));}}}
        self.native_world=Some(state);
        //420200 enables both41cab0 lookup flags and probes origin+(16,24).
        //41cb40 and OBJECT/terrain retirement occur only for that first hit.
        if let Some((blocker,point,object_slot))=self.native_ambient_blocker([origin[0].wrapping_add(16),origin[1].wrapping_add(24)]){
            self.native_ambient_terrain_retired(point);
            if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(usize::from(object_slot))).and_then(Option::as_mut){o.active=false;o.record.position[0]=-1;if let Some(raw)=o.render.and_then(|i|self.objects.get_mut(i)){raw["nativeActive"]=json!(false);}}
            let cell=Self::world_cell(point);self.set_world_code(cell,self.world_code(cell)&128);self.retire_native_ambient_blocker(blocker);
            let world=self.map.wall_tile_lookup["nativeWorld"].as_u64().map(|n|n as usize).unwrap_or([0,3,1,4,2][self.map.world.min(4)]);let sprite=[2951,2945,2948,2957,2954][world.min(4)];for delta in 0..3{self.spawn_native_world_particles(effects::Kind::FallingObject(sprite+delta),[point[0].wrapping_sub(160),point[1].wrapping_sub(92)],1);}
        }
        if defer_terrain{self.sync_native_world();return;}
        if strength!=0&&!self.native_world.as_ref().is_some_and(|s|s.effect_world_mask4f!=0){let tint=self.world_code(Self::world_cell([origin[0].wrapping_add(12),origin[1].wrapping_add(32)]))&127;if let Some(state)=&mut self.native_world{state.effect_tint=tint;}}
        match native::environment_contact(origin,fire_mode,|cell|self.world_code(cell)) {
            Some(native::TerrainContact::Wall(cell))=>{self.damage_native_wall(cell,strength,team);}
            Some(native::TerrainContact::Bridge(cell))=>{let point=[(cell&127)*16,(cell>>7)*16];self.damage_native_bridge(point,strength,team);},None=>{}
        }
        self.sync_native_world();
    }
    fn damage_native_world_object(&mut self,slot:usize,strength:u8,sample:[i16;2]){
        self.damage_native_world_object_mode(slot,strength,sample,false);
    }
    fn damage_native_world_object_mode(&mut self,slot:usize,strength:u8,sample:[i16;2],force:bool){
        let Some(mut state)=self.native_world.take()else{return;};let Some(Some(object))=state.objects.get_mut(slot)else{self.native_world=Some(state);return;};
        let(hp,dead)=native::object_damage(object.hp,i16::from(strength),force);object.hp=hp;let snapshot=object.clone();let seasonal=dead&&self.native_seasonal_mode==1&&snapshot.sprite==5960&&snapshot.record.metadata_flags18&12==12;if dead&&!seasonal{object.active=false;}
        if dead&&!seasonal{if let Some(index)=snapshot.render{if let Some(v)=self.objects.get_mut(index){v["nativeActive"]=json!(false);}}}
        if let Some(id)=snapshot.unit{if let Some(u)=self.units.iter_mut().find(|u|u.id==id){u.hp=if dead{-1.}else{f64::from(hp)*25.};if dead&&snapshot.kind!=31{u.cargo.clear();}}}
        if dead{if let Some(id)=snapshot.pickup{if let Some(c)=self.crates.get_mut(id){c.taken=true;}}}
        self.native_world=Some(state);self.spawn_native_object_particles(effects::Kind::Hit,sample);if !dead{return;}
        self.spawn_native_object_particles(effects::Kind::Destroyed,sample);
        if snapshot.kind==31{self.remove_native_apc_object_occupants(slot as u8);}
        let base=Self::world_cell(snapshot.record.position);
        if native::object_secondary_explosion(snapshot.sprite){let old=self.world_code(base);self.set_world_code(base,(old&0x96)|22);if let Some(s)=&mut self.native_world{s.suppress_objects=s.suppress_objects.saturating_add(1);}
            self.spawn_native_blast(i32::from(snapshot.record.position[0])-16,i32::from(snapshot.record.position[1])-16,Attack{target:0,damage:127,aim:0,facing:0,bullet:false},255,None);
            if let Some(s)=&mut self.native_world{s.suppress_objects=s.suppress_objects.saturating_sub(1);}
            self.spawn_native_world_particles(effects::Kind::CrateBurst,[snapshot.record.position[0].wrapping_sub(176),snapshot.record.position[1].wrapping_sub(108)],if snapshot.sprite==350{10}else{20});
        }
        if snapshot.record.metadata_flags18&4!=0 {let cell=Self::world_cell([snapshot.record.position[0],snapshot.record.position[1].wrapping_add(if snapshot.record.metadata_flags18&2!=0{16}else{0})]);let old=self.world_code(cell);if old&127!=7{self.set_world_code(cell,(old&0x96)|22);}}
        else if native::restores_wreck_tiles(snapshot.kind){for(i,delta)in[0,1,128,129].into_iter().enumerate(){let cell=base.wrapping_add(delta);let old=self.world_code(cell);self.set_world_code(cell,(old&128)|native::restored_wreck_terrain(snapshot.saved[i]));}}
        if snapshot.record.metadata_flags18&12==12{self.emit_native_object_falling(snapshot.record.position,snapshot.sprite as i16);}
        if seasonal{let point=super::original_display::seasonal::destruction_point(snapshot.record.position,snapshot.record.metadata_flags18);if let Err(e)=self.emit_native_seasonal_particles(point){self.native_route_boundary=Some(e);}if let Some(o)=self.native_world.as_mut().and_then(|s|s.objects.get_mut(slot)).and_then(Option::as_mut){o.active=false;o.record.position[0]=-1;}if let Some(raw)=snapshot.render.and_then(|i|self.objects.get_mut(i)){raw["nativeActive"]=json!(false);}}
        for delta in [0,1,128,129]{let cell=base.wrapping_add(delta);let old=self.world_code(cell);self.set_world_code(cell,native::clear_object_terrain(old));}
        self.map.revision=self.map.revision.wrapping_add(1);
    }
    fn damage_native_bridge(&mut self,point:[i16;2],strength:u8,team:i32){
        let Some(mut state)=self.native_world.take()else{return;};
        if let Some(index)=state.bridges.iter().position(|b|b.native_slot<12&&native::bridge_contains(b.record,point)){let bridge=&mut state.bridges[index];let(hp,flags,dead,half)=native::bridge_damage(bridge.record,i16::from(strength));bridge.record.hp=hp;bridge.record.flags15=flags;
            if let Some(value)=self.map.bridge_hp.get_mut(bridge.binding){*value=f64::from(hp.max(0));}
            let record=bridge.record;let orientation=bridge.orientation;let binding=bridge.binding;
            if half||dead{self.native_bridge_particle_prefix(&mut state,record,orientation,half);}
            if half{self.events.push(json!({"kind":"bridgeDamaged","id":binding}));}
            if dead{self.native_bridge_collapse_sound(record.position);let rect=native::bridge_collapse_rectangle(record,orientation);self.native_world=Some(state);self.collapse_native_units(rect,true);let Some(updated)=self.native_world.take()else{return;};state=updated;
                for object in state.objects.iter_mut().flatten().filter(|o|o.active&&native::collapse_object(o.kind,o.record.position,rect)){
                    let base=Self::world_cell(object.record.position);for(i,delta)in[0,1,128,129].into_iter().enumerate(){let cell=base.wrapping_add(delta);let old=self.world_code(cell);self.set_world_code(cell,(old&128)|native::restored_wreck_terrain(object.saved[i]));}
                    object.active=false;if let Some(id)=object.unit{if let Some(u)=self.units.iter_mut().find(|u|u.id==id){u.hp=-1.;}}
                    if let Some(index)=object.render{if let Some(v)=self.objects.get_mut(index){v["nativeActive"]=json!(false);}}
                }
                let bridge=&mut state.bridges[index];self.collapse_native_bridge_terrain(bridge);
                for j in 0..self.map.tiles.len(){if self.map.bridge_base.get(j)==Some(&(bridge.binding as i32)){self.map.tiles[j]=3;let code=self.map.water_tile;self.map.original_tiles[j]=code;}}
                self.map.revision=self.map.revision.wrapping_add(1);self.events.push(json!({"kind":"bridgeDestroyed","id":bridge.binding,"team":team}));
            }
        }
        self.native_world=Some(state);
    }
    fn collapse_native_units(&mut self,rect:[i16;4],bridge_mode:bool){
        for index in 0..self.units.len(){let u=&self.units[index];
            let flags=u.native_slot.and_then(|slot|self.native_group_rosters.members.get(slot)).and_then(Option::as_ref).map(|m|m.flags31).unwrap_or_else(||if original_combat::alive(true,u.hp)&&u.team>=0&&u.transport_carrier.is_none()&&!super::workshops::hidden(u){4}else{0});
            if native::collapse_unit(flags,u.native_class7c,native_point(u.x,u.y),rect,bridge_mode){let before=u.clone();self.units[index].hp=-1.;self.units[index].cargo.clear();self.native_contact_removal(index,&before,super::original_death::RemovalSource::Collapse,false);}
        }
    }
    fn damage_native_wall(&mut self,cell:i16,strength:u8,team:i32){
        let(head,_height)=native::wall_anchor(cell,|cell|self.world_code(cell));
        let Some(index)=self.walls.iter().take(5000).position(|w|w.alive()&&Self::world_cell(native_point(w.x,w.y))==head)else{return;};
        let w=&mut self.walls[index];let hp=w.native_hp.unwrap_or(w.hp as i16)as i8;let(hp,dead)=native::wall_damage(hp,strength);w.native_hp=Some(i16::from(hp));
        if !dead{w.hp=f64::from(hp);return;}
        let id=w.id;let amount=w.hp.max(1.);self.damage_wall(id,amount,team);
        //The wall renderer regenerates both native planes, including sprite
        //bank bits. Publish those writes to the full128-column terrain plane.
        for y in 0..self.map.height{for x in 0..self.map.width{let j=y*self.map.width+x;let code=(self.map.terrain_codes[j]&127)|(((self.map.original_tiles[j]&256)>>1)as u8);if let Some(v)=self.map.native_terrain_plane.get_mut(y*128+x){*v=code;}}}
    }
    fn collapse_native_bridge_terrain(&mut self,bridge:&mut BridgeSlot){
        let(changes,state14)=native::collapse_bridge_terrain(bridge.record,bridge.orientation,|cell|self.world_code(cell));bridge.raw_tail[14]=state14;
        for(cell,code)in changes{if !bridge.restored_terrain.iter().any(|&(old,_)|old==cell){bridge.restored_terrain.push((cell,self.world_code(cell)));}self.set_world_code(cell,code);}
        if let Some((index,bounds))=self.map.native_road_graph.as_ref().and_then(|g|g.region_at((bridge.record.position[0],bridge.record.position[1])).map(|slot|(usize::from(slot),g.regions[usize::from(slot)].bounds))){
            let changes=native::collapsed_road_terrain(bounds,|cell|self.world_code(cell));for(cell,code)in changes{if !bridge.restored_terrain.iter().any(|&(old,_)|old==cell){bridge.restored_terrain.push((cell,self.world_code(cell)));}self.set_world_code(cell,code);}
            bridge.retired_road=Some((index,bounds));if let Some(g)=&mut self.map.native_road_graph{g.regions[index].bounds=[0;4];}
        }
    }
    fn spawn_native_object_particles(&mut self,kind:effects::Kind,sample:[i16;2]){
        self.spawn_native_world_particles(kind,[sample[0].wrapping_sub(154),sample[1].wrapping_sub(80)],16);
    }
    fn spawn_native_world_particles(&mut self,kind:effects::Kind,origin:[i16;2],count:u8){
        self.spawn_native_owned_particles(kind,origin,count,0);
    }
    fn spawn_native_owned_particles(&mut self,kind:effects::Kind,origin:[i16;2],count:u8,owner:u8){
        let Some(state)=&mut self.native_world else{return;};state.effect_owner=owner;match kind{effects::Kind::Destroyed=>state.effect_maximum=5,effects::Kind::FallingObject(_)|effects::Kind::CrateBurst=>state.effect_maximum=0,effects::Kind::Poof{maximum,..}|effects::Kind::Fragment{maximum,..}=>state.effect_maximum=maximum,_=>{}}
        for _ in 0..count{let Some(slot)=state.effects.iter().position(|p|p.as_ref().is_none_or(|p|!p.active))else{break;};let mut p=effects::initialize(kind,origin,state.effect_tint,||self.original_rng.next_u32());p.owner=owner;p.maximum_phase=state.effect_maximum;state.put_effect(slot,p);}
    }
    pub(crate) fn spawn_native_ambient_fragment(&mut self,point:[i16;2],sprite:i16){let owner=self.native_world.as_ref().map_or(0,|s|s.effect_owner);self.spawn_native_owned_particles(effects::Kind::AmbientFragment(sprite),[point[0].wrapping_sub(160),point[1].wrapping_sub(92)],1,owner);}
    ///402670 intentionally constructs a hit particle and immediately returns
    ///the same slot to the pool. It is a seven-RNG write, not a visible poof.
    pub(crate) fn native_lava_burst(&mut self,point:[i16;2]){let Some(slot)=self.native_world.as_ref().and_then(|s|s.effects.iter().position(|p|p.as_ref().is_none_or(|p|!p.active)))else{return;};self.original_rng.next_u32();self.spawn_native_owned_particles(effects::Kind::Hit,[point[0].wrapping_sub(154),point[1].wrapping_sub(80)],1,0);if let Some(p)=self.native_world.as_mut().and_then(|s|s.effects[slot].as_mut()){p.active=false;p.position[0]=-1000;}}
    pub(crate) fn native_removal_emission(&mut self,kind:super::original_death::EmitterKind,origin:[i16;2],count:u8,sprite:i16,maximum:i8,owner:u8){
        if self.native_world.is_none(){self.sync_native_world();}let kind=match kind{super::original_death::EmitterKind::Destroyed=>effects::Kind::Destroyed,super::original_death::EmitterKind::Fragment=>effects::Kind::Fragment{sprite,maximum},super::original_death::EmitterKind::Poof=>effects::Kind::Poof{sprite,maximum}};self.spawn_native_owned_particles(kind,origin,count,owner);
    }
    pub(crate) fn native_removal_sound(&mut self,point:[i16;2],id:u8){if self.native_world.is_none()||self.native_blast_sound_gate{return;}self.native_blast_sound_gate=true;if id==6{self.events.push(json!({"kind":"nativeEffectSound","sound":6,"sample":"GYRO5","x":i32::from(point[0])*2,"y":i32::from(point[1])*2}));}}
    ///58af0/58a40 convert the released hardware into the same first-free
    ///OBJECT pool as authored hardware, flags, fires and pickups.
    pub(crate) fn native_removed_hardware(&mut self,index:usize,before:&super::Unit,suppressed:bool,crew_only:bool){
        self.native_removed_hardware_mode(index,before,suppressed,crew_only,self.multiplayer);
    }
    ///59720 sets4a6e77 before58af0: an unload preserves recoverable hardware
    ///in multiplayer too. It does not call a contact-death emitter/removal.
    pub(crate) fn park_native_hardware_for_unload(&mut self,index:usize,before:&super::Unit){
        self.sync_native_world();self.native_removed_hardware_mode(index,before,false,true,false);
    }
    fn native_removed_hardware_mode(&mut self,index:usize,before:&super::Unit,suppressed:bool,crew_only:bool,forced_wreck:bool){
        use super::original_death as death;
        if stats(&before.kind).cls==0{return;}
        if let Some(slot)=before.native_slot{self.remember_native_point_target(slot,native_point(before.x,before.y),before.native_class7c);}
        let family=original_combat::animation_family(&before.kind);let class=before.native_class7c;let point=native_point(before.x,before.y);let owner=before.native_flags64;
        let mut hull=self.units[index].hp as i16;
        if class==2&&self.native_fort_bases().is_some_and(|bases|bases.into_iter().any(|p|point[0]>=p[0]&&point[0]<=p[0].wrapping_add(160)&&point[1]>=p[1].wrapping_sub(16)&&point[1]<=p[1].wrapping_add(184))){hull=-1;self.units[index].hp=-1.;}
        let converted=death::conversion(death::ConversionInput{family,class,hull,position:point,owner,suppressed,crew_only,removing:suppressed,forced_wreck});
        let Some(wreck)=converted.object else{return;};
        let cell=Self::world_cell([wreck.position[0].wrapping_add(7),wreck.position[1].wrapping_add(7)]);let terrain=[0,1,128,129].map(|o|self.world_code(cell.wrapping_add(o)));
        if !death::insertion_allowed(wreck.kind,terrain){
            let aligned=wreck.position.map(|v|v.wrapping_add(7)&!15);self.native_removal_emission(death::EmitterKind::Destroyed,[aligned[0].wrapping_sub(154),aligned[1].wrapping_sub(80)],64,557,5,owner);self.units[index].hp=-1.;return;
        }
        let Some(mut state)=self.native_world.take()else{return;};let Some(slot)=state.free()else{self.native_world=Some(state);self.units[index].hp=-1.;return;};
        if let Some(render)=state.objects[slot].as_ref().and_then(|o|o.render).and_then(|i|self.objects.get_mut(i)){render["nativeActive"]=json!(false);}
        let old=state.objects[slot].as_ref();let latent=if slot==25{state.retired_workshop_alias25}else{None};let old_flags26=old.map_or_else(||latent.map_or(0,|a|a.word1a.to_le_bytes()[0]),|o|o.flags26);let old_variant26=old.map_or_else(||latent.map_or(0,|a|a.byte26 as i8),|o|o.variant26);let old_saved=old.map_or([0;4],|o|o.saved);
        let hp=death::object_hull(hull);let Some(mut object)=self.make_object(wreck.kind,wreck.position,hp)else{self.native_world=Some(state);return;};object.maximum=(before.max_hp as i16/25)as u8;object.record.flags13=wreck.flags13;object.flags19=if wreck.wreck{0}else{15};object.flags26=old_flags26;object.variant26=if wreck.wreck{(self.original_rng.next_u32()&3)as i8}else{old_variant26};object.lifetime24=if wreck.wreck{288}else{0};object.saved=old_saved;object.unit=(!wreck.wreck).then_some(before.id);object.bucket12=native_bucket(wreck.position);
        let equipment=(0..19u8).find(|&eq|equipment_kind(u16::from(eq))==Some(before.kind.as_str()));object.hardware=equipment.and_then(super::original_entry::equipment_attributes);
        if let Some(h)=self.units[index].native_hardware_slot.and_then(|s|self.native_hardware_pool.get(usize::from(s))).map(|s|s.attributes){let mut payload=super::original_entry::ObjectHardware::default();for(to,from)in[(14,5),(17,6),(16,7),(24,32),(25,33),(27,35),(28,36),(29,37),(30,38)]{payload.set_byte(to,h.byte(from));}if wreck.kind==31{payload.set_byte(17,0);}object.maximum=(h.word(10)/25)as u8;object.hardware=Some(payload);}
        else if let Some(payload)=&mut object.hardware{if let Some(c)=&self.units[index].original_combat{payload.set_byte(16,c.movement_control);payload.set_byte(24,c.fire_rate);if let Some(speed)=c.projectile_speed{payload.set_byte(25,speed);}}}
        let mut contacts=Vec::new();
        if wreck.align{
            let aligned=wreck.position.map(|v|v.wrapping_add(7)&!15);let overlap=death::road_waypoint_overlap(aligned,self.map.native_road_graph.as_ref().map(|g|g.regions.iter().map(|r|r.waypoint).collect::<Vec<_>>()).unwrap_or_else(||vec![0;96]));
            let units=self.units.iter().enumerate().map(|(i,u)|{let mut position=native_point(u.x,u.y);if u.native_transport.as_ref().is_some_and(|a|a.phase==super::original_transport::Phase::Inside){position[0]=10000;}let flags31=u.native_slot.and_then(|s|self.native_group_rosters.members.get(s)).and_then(Option::as_ref).map_or_else(||if u.team>=0&&original_combat::alive(true,u.hp){4}else{0},|m|m.flags31);death::PlacementUnit{position,family:original_combat::animation_family(&u.kind),flags31,phase35:u.native_phase35,self_unit:i==index}}).collect::<Vec<_>>();
            let plan=death::place_wreck(wreck.position,terrain,None,&units,overlap);object.record.position=plan.position;if let Some(saved)=plan.saved{object.saved=saved;for(i,delta)in[0,1,128,129].into_iter().enumerate(){self.set_world_code(cell.wrapping_add(delta),plan.terrain[i]);}}contacts=plan.contacts;
        }
        let render=self.objects.len();let eq=if wreck.wreck{None}else{(0..19u16).find(|&eq|equipment_kind(eq)==Some(before.kind.as_str()))};let p=object.record.position;
        self.objects.push(json!({"type":wreck.kind,"equipmentId":eq,"x":i32::from(p[0])*2,"y":i32::from(p[1])*2,"nativeSlot":slot,"nativeActive":true,"nativeHp":hp,"nativeFlags13":object.record.flags13,"nativeFlags19":object.flags19,"nativeFlags26":object.flags26,"nativeTeam":owner&3,"templateSprite":object.sprite}));object.render=Some(render);object.workshop_raw1b=object.hardware.map(|h|h.byte(27)).or_else(||old.and_then(|o|o.workshop_raw1b)).or_else(||latent.map(|a|a.word1a.to_le_bytes()[1])).or(Some(0));if slot==25{state.retired_workshop_alias25=None;}state.objects[slot]=Some(object);self.native_world=Some(state);
        //58af0 clears UNIT+8 after successful OBJECT allocation and frees the
        //saved hardware record itself.58660 therefore sees no hardware pointer.
        self.release_native_hardware_record(index);self.units[index].native_hardware_slot=None;
        if wreck.wreck{self.units[index].hp=-1.;}else{self.units[index].x=f64::from(p[0])*2.;self.units[index].y=f64::from(p[1])*2.;self.units[index].hp=f64::from(hp)*25.;}
        self.map.revision=self.map.revision.wrapping_add(1);
        for(origin,strength)in contacts{let selected=self.native_world.as_ref().and_then(|s|{let objects=s.objects.iter().map(|o|o.as_ref().filter(|o|o.active).map(|o|o.record).unwrap_or(native::Object{position:[-1,-1],flags13:0,metadata_flags18:0,metadata_extent20:0})).collect::<Vec<_>>();native::blast_object(&objects,origin,true,false)});if let Some((slot,sample))=selected{let d=[[4,16],[20,16],[20,32],[4,32]][sample];self.damage_native_world_object(slot,strength,[origin[0].wrapping_add(d[0]),origin[1].wrapping_add(d[1])]);}}
    }
    ///The source graphics setting selects500,1000,1500 or2000 slots. Quality0
    ///is the default here; the menu may set another original quality explicitly.
    pub(crate) fn set_native_world_effect_quality(&mut self,quality:u8){if let Some(state)=&mut self.native_world{state.effect_quality=quality.min(3)as i8;state.effects.resize_with((usize::from(quality.min(3))+1)*500,||None);}}
    ///Optional original viewport adapter, in raw native pixels. Without a local
    ///viewport the simulation advances visible phases across the revealed map.
    pub(crate) fn set_native_world_effect_view(&mut self,camera:[i16;2],clip:[i16;4]){if let Some(state)=&mut self.native_world{state.effect_view=Some((camera,clip));}}
    pub(crate) fn update_native_world_effects(&mut self){
        let Some(state)=&mut self.native_world else{return;};let(camera,clip)=state.effect_view.unwrap_or(([0,0],[-32768,-32768,32767,32767]));let view=effects::View{camera,clip,world:[(self.map.width*16)as i16,(self.map.height*16)as i16],sine:super::native_math::sine(425),cosine:super::native_math::sine(553)};
        state.effect_draws.clear();let length=state.effects.len();
        for slot in 0..length{let mut resume=false;loop{
            let Some(state)=&mut self.native_world else{return;};let Some(p)=state.effects[slot].as_mut().filter(|p|p.active)else{break;};
            let step=if resume{p.resume_native_trail(view,&mut state.last_projection,&mut state.trail)}else{p.advance_native_trail(view,&mut state.last_projection,&mut state.trail)};
            if let Some(draw)=step.draw{if draw.primitive.is_none()&&!draw.inherited_palette{state.direct_palette=Some(draw.owner&3);}state.effect_draws.push((slot,draw,state.effect_cp_tints.get(slot).copied().flatten()));}let Some(mode)=step.expiry else{break;};let point=[p.origin[0].wrapping_add(state.last_projection[0]),p.origin[1].wrapping_add(state.last_projection[1])];let actions=effects::expiry_prefix_with_sprite(mode,point,p.sprite,view,self.native_blast_sound_gate);
            if matches!(mode,5|8){self.retire_native_fragment_water_particle(slot);break;}
            if mode==3{self.finish_native_building_part_expiry(slot,view);break;}
            if mode==11{self.finish_native_bridge_expiry(slot);break;}
            self.execute_native_effect_actions(slot,actions,view);
            resume=self.native_world.as_ref().and_then(|s|s.effects[slot].as_ref()).is_some_and(|p|p.active&&p.position[2]>=0);if !resume{break;}
        }}
    }
    fn execute_native_effect_actions(&mut self,slot:usize,actions:Vec<effects::Action>,view:effects::View){
        let mut pending=VecDeque::from(actions);while let Some(action)=pending.pop_front(){match action{
            effects::Action::Blast{point,strength,suppress_visual}=>{if suppress_visual{self.native_blast_environment(i32::from(point[0]),i32::from(point[1]),strength,255,false);}else{let before=self.events.len();let quiet=self.native_world.as_ref().is_some_and(|s|s.quiet_blasts);self.spawn_native_blast(i32::from(point[0]),i32::from(point[1]),Attack{target:0,damage:strength,aim:0,facing:0,bullet:false},255,None);if quiet{for event in self.events.iter_mut().skip(before).filter(|e|e["kind"]=="explosion"){event["silent"]=json!(true);}}}}
            effects::Action::DirectUnits{rect,strength}=>self.native_direct_environment_contact(rect,strength),
            effects::Action::Spawn{kind,origin,count}=>self.spawn_native_world_particles(kind,origin,count),
            effects::Action::RandomDestroyedBurst{origin}=>{let count=(self.original_rng.next_u32()as u8&3)+5;self.spawn_native_world_particles(effects::Kind::Destroyed,origin,count);}
            effects::Action::RandomHitBurst{origin}=>{let count=(self.original_rng.next_u32()as u8&3)+4;self.spawn_native_world_particles(effects::Kind::Hit,origin,count);}
            effects::Action::Sound{point,id}=>{let sample=match id{30=>"GRENADE",11=>"METGRND",8=>match self.map.world{1=>"VOLCGLG2",3=>"MUDPLOP1",_=>"SPLASH1"},_=>continue};self.events.push(json!({"kind":"nativeEffectSound","sound":id,"sample":sample,"x":i32::from(point[0])*2,"y":i32::from(point[1])*2}));}
            effects::Action::SetCrateSoundGate=>{self.native_blast_sound_gate=true;}
            effects::Action::QuietBlasts=>{if let Some(state)=&mut self.native_world{state.quiet_blasts=true;}}
            effects::Action::ClearQuietBlasts=>{if let Some(state)=&mut self.native_world{state.quiet_blasts=false;}}
            effects::Action::ProbeFalling{point}|effects::Action::ProbeCrate{point}=>{let mode=if matches!(action,effects::Action::ProbeFalling{..}){7}else{6};let terrain=self.world_code(Self::world_cell(point));let suffix=effects::expiry_suffix(mode,point,terrain,view);for a in suffix.into_iter().rev(){pending.push_front(a);}}
            effects::Action::ProbeAmbient{point}=>{let terrain=self.world_code(Self::world_cell(point));for a in effects::ambient_suffix(point,terrain).into_iter().rev(){pending.push_front(a);}}
            effects::Action::ProbeFragment{point,sprite}=>{let terrain=self.world_code(Self::world_cell(point));let gate=self.native_blast_sound_gate;for a in effects::fragment_suffix(point,terrain,sprite,view,gate).into_iter().rev(){pending.push_front(a);}}
            effects::Action::Retire{..}|effects::Action::InvalidCrate|effects::Action::ResidualCrate=>{if let Some(p)=self.native_world.as_mut().and_then(|s|s.effects[slot].as_mut()){effects::apply_particle_action(p,&action);}}
        }}
    }
    fn native_direct_environment_contact(&mut self,rect:[i16;4],strength:u8){
        let suppress=self.native_world.as_ref().is_some_and(|s|s.effect_world_mask4f!=0);self.native_direct_environment_contact_mode(rect,strength,suppress);
    }
    fn native_direct_environment_contact_mode(&mut self,rect:[i16;4],strength:u8,suppress_hardware_particles:bool){
        self.native_ambient_area_contact([rect[0],rect[1]]);
        let coarse=(rect[1]>>3&!15).wrapping_add(rect[0]>>7);
        //456640 preserves this nine-bucket order; each bucket was populated in
        //native UNIT slot order and capped at32 before rectangle filtering.
        for offset in[-17i16,-16,-15,-1,0,1,15,16,17]{let bucket=coarse.wrapping_add(offset);if !(0..256).contains(&bucket){continue;}
            let mut ids=self.units.iter().filter(|u|u.team>=0&&original_combat::alive(true,u.hp)).filter(|u|{let p=native_point(u.x,u.y);(p[1]>>3&!15).wrapping_add(p[0]>>7)==bucket}).map(|u|(u.native_slot.unwrap_or(u.id),u.id)).collect::<Vec<_>>();ids.sort_unstable();
            for(_,id)in ids.into_iter().take(32){let Some(u)=self.units.iter().find(|u|u.id==id)else{continue;};let class=u.native_class7c;let point=native_point(u.x,u.y);let member=u.native_slot.and_then(|slot|self.native_group_rosters.members.get(slot)).and_then(Option::as_ref);let flags=member.map(|m|m.flags31).unwrap_or(4);let phase=u.native_phase35;
                if flags&4==0||class!=1&&phase==18||!effects::direct_unit_contains(point,class,rect){continue;}
                if class!=1||!suppress_hardware_particles{self.spawn_native_world_particles(if class==1{effects::Kind::Hit}else{effects::Kind::DirectHit},[point[0].wrapping_sub(146),point[1].wrapping_sub(82)],2);}
                self.apply_native_contact_from(Attack{target:id,damage:strength,aim:0,facing:0,bullet:false},super::original_death::RemovalSource::Direct);
            }
        }
    }
    pub(crate) fn native_world_effects(&self)->Vec<Value>{
        self.native_world.as_ref().map(|s|s.effect_draws.iter().map(|&(id,d,cp_owner)|{
            if let Some(effects::Primitive::ColorPoint{color})=d.primitive{let mut row=json!({"id":id,"primitive":"colorPoint","colorIndex":color,"x":i32::from(d.point[0])*2,"y":i32::from(d.point[1])*2,"nativeDrawX":d.screen[0],"nativeDrawY":d.screen[1]});if let Some(owner)=cp_owner{row["nativeTintOwner"]=json!(owner);}row}
            else{let mut row=json!({"id":id,"sprite":d.sprite,"x":i32::from(d.point[0])*2,"y":i32::from(d.point[1])*2,"nativeDrawX":d.screen[0],"nativeDrawY":d.screen[1],"scale":if d.transformed{f64::from((f32::from(d.height)+16.)*0.03125)}else{1.},"rotation":if d.transformed{f64::from((f32::from(d.angle)*2.*3.1415925f32)*0.00390625)}else{0.}});if d.inherited_palette{row["source"]=json!("native4666c0");row["nativeInheritedPalette"]=json!(true);}else{row["team"]=json!(d.owner&3);row["nativeLiteralPalette"]=json!(d.owner&4!=0);}row}
        }).collect()).unwrap_or_default()
    }
    pub(crate) fn native_blast_quiet(&self)->bool{self.native_world.as_ref().is_some_and(|s|s.quiet_blasts)}
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Sector,Crate,original_group_game,original_route};
    pub(super) fn field()->Game{
        let mut g=Game::new(0,1,5271);let(w,h)=(64,64);g.map.width=w;g.map.height=h;g.map.original=true;
        g.map.tiles=vec![0;w*h];g.map.original_tiles=vec![0;w*h];g.map.terrain_codes=vec![0;w*h];g.map.native_terrain_plane=vec![0;16384];g.map.territory=vec![0;w*h];g.map.bridge_base=vec![-1;w*h];g.map.bridge_hp.clear();g.map.bridge_max_hp.clear();g.map.native_road_graph=None;g.map.native_level_record=0;
        g.units.clear();g.buildings.clear();g.objects.clear();g.crates.clear();g.walls.clear();g.fires.clear();g.blasts.clear();g.events.clear();g.native_strategy=None;g.native_world=None;g.native_group_rosters=original_group_game::State::default();g.winner=None;g.next_id=1;
        g.sectors=vec![Sector{id:0,x:32.,y:32.,owner:0,home:true,counts_for_production:true}];g
    }
    #[test]
    fn classified_attack_uses_stale_argument_y_for_the_source_object_marker() {
        let mut g=field();g.spawn("grunt",0,640.,640.,1,None);g.maintain_native_group_rosters();
        g.native_world=Some(State::empty());
        let mut object=g.make_object(28,[300,100],0).unwrap();
        object.active=false;object.record.flags13=0;object.record.metadata_flags18=16;
        g.native_world.as_mut().unwrap().objects[7]=Some(object);
        let rng=g.original_rng.words;
        let event=super::super::original_orders_game::Event::classified(5,[200,305],105);
        assert!(g.record_native_order_event(0,event));
        assert_eq!(g.units[0].native_fieldb3,1,"43ea80 uses old eventY/metadata, not the enemy UNIT position, active bit or flags13");
        assert_eq!(g.original_rng.words,rng);
    }
    #[test]
    fn order_markers_match_512_actual_classified_native_callers() {
        use super::super::original_orders_game::Event;
        let offsets=[-1i16,0,7,8,15,16,23,24,32767,-32768];
        let mut signature=2166136261u32;
        for seed in 0usize..512 {
            let mut g=field();g.spawn("grunt",0,640.,640.,1,None);g.maintain_native_group_rosters();
            g.native_world=Some(State::empty());g.units[0].native_fieldb3=99;
            g.native_world.as_mut().unwrap().order_object_pointer=0x12340000+seed as u32;
            for (i,p) in [[300,100],[301,101],[-2,-2],[32760,32760]].into_iter().enumerate() {
                let mut o=g.make_object(28,p,0).unwrap();o.active=seed&1!=0;o.hp=0;
                o.record.flags13=(seed*37)as u8;
                o.record.metadata_flags18=if i==0{(seed*13)as u8}else{16+((seed+i)%4)as u8};
                g.native_world.as_mut().unwrap().objects[i]=Some(o);
            }
            let op=[2,5,4,8,9,11,12,13][seed%8];
            let q=[300i16.wrapping_add(offsets[seed/8%10]),100i16.wrapping_add(offsets[seed/80%6])];
            let event=Event::classified(op,if op==5{[200,q[0]]}else{q},q[1]);
            let rng=g.original_rng.words;assert!(g.record_native_order_event(0,event));
            assert_eq!(g.original_rng.words,rng);
            let pointer=g.native_world.as_ref().unwrap().order_object_pointer;
            for n in [seed as u32,op as u32,u32::from(g.units[0].native_fieldb3 as u8),pointer] {
                signature=(signature^n).wrapping_mul(16777619);
            }
            //A query miss preserves the pointer even across a real world import.
            g.objects.clear();g.import_native_world(&Value::Null);
            assert_eq!(g.native_world.as_ref().unwrap().order_object_pointer,pointer);
            assert_eq!(g.native_order_object_marker(Event::classified(2,[-100,-100],0)),Some(-1));
            assert_eq!(g.native_world.as_ref().unwrap().order_object_pointer,pointer);
        }
        assert_eq!(signature,3488280415,"512 actual45f070 C3=1 callers through43ea80; B3/pointer projection, not whole router");
    }
    fn building(id:usize,kind:&str,x:f64,y:f64,hp:f64)->Building{
        Building{id,x,y,w:128.,h:160.,kind:kind.into(),team:0,sector:0,stars:0,hp,max_hp:hp,product:None,progress:0.,ready_guns:vec![],gate:None,orientation:String::new(),remaining_work:None,completion_ticks:0,raw_base:None}
    }
    fn neutral(g:&mut Game,kind:&str,x:f64,y:f64,hp:f64)->usize{g.spawn(kind,-1,x,y,0,None);let u=g.units.last_mut().unwrap();u.hp=hp;u.crew=None;u.id}
    fn raw(kind:u16,eq:u16,slot:usize,x:i32,y:i32,hp:u8)->Value{json!({"type":kind,"equipmentId":eq,"nativeSlot":slot,"x":x,"y":y,"nativeHp":hp})}
    #[test]fn raw_claim_goal_distinguishes_cold_object_slots_from_retired_and_prior_mission_slots(){let mut g=field();g.objects=vec![raw(31,255,0,400,600,0)];g.import_native_world(&Value::Null);assert_eq!(g.native_object_raw_position(199),Some([-1,0]));assert_eq!(g.native_object_raw_position(200),None);assert_eq!(g.native_object_raw_position(0),Some([200,300]));g.apply_native_ambient_object(0,[-1,300],0);assert_eq!(g.native_object_raw_position(0),Some([-1,300]));g.objects.clear();g.import_native_world(&Value::Null);assert_eq!(g.native_object_raw_position(0),Some([-1,300]));assert_eq!(g.native_object_raw_position(199),Some([-1,0]));assert!(g.native_world_scan_objects().is_empty());assert!(g.native_object_visuals().is_empty());}
    #[test]fn authored_apc_render_offset_and_portal_writer_are_separate_from_animation(){let mut g=field();g.objects=vec![raw(31,255,0,400,400,0)];g.import_native_world(&Value::Null);assert_eq!(g.native_object_sprite_offset(0),Some(30));assert_eq!(g.objects[0]["nativeSpriteOffset21"],30);g.set_native_object_animation(0,3,1);g.set_native_object_sprite_offset(0,10);g.motion_tick=1;g.update_native_world_objects();assert_eq!(g.objects[0]["nativeSpriteOffset21"],10);assert_eq!(g.objects[0]["nativeAnimation18"],3);assert_eq!(g.native_object_sprite_offset(0),Some(10));}
    #[test]fn lava_burst_consumes_seven_words_and_immediately_releases_its_shared_particle_slot(){let mut g=field();g.import_native_world(&Value::Null);g.native_world.as_mut().unwrap().effect_maximum=3;let mut expected=g.original_rng.clone();for _ in 0..7{expected.next_u32();}g.native_lava_burst([320,320]);assert_eq!(g.original_rng.words,expected.words);let p=g.native_world.as_ref().unwrap().effects[0].as_ref().unwrap();assert!(!p.active);assert_eq!((p.position[0],p.origin,p.owner,p.maximum_phase),(-1000,[166,240],0,3));assert!(g.native_world.as_ref().unwrap().effect_draws.is_empty());g.spawn_native_object_particles(effects::Kind::Hit,[320,320]);g.native_world.as_mut().unwrap().effects.truncate(1);let before=g.original_rng.words;g.native_lava_burst([320,320]);assert_eq!(g.original_rng.words,before);}
    #[test]fn one_object_fire_pulse_can_retire_a_later_slot_before_its_delay_and_expiry_callbacks(){let mut g=field();g.motion_tick=0;g.native_fire_base_strength=20;let mut wreck=raw(6,255,1,408,432,1);wreck["nativeFlags13"]=json!(0x70);g.objects=vec![raw(73,255,0,400,400,0),wreck];g.import_native_world(&Value::Null);{let state=g.native_world.as_mut().unwrap();let fire=state.objects[0].as_mut().unwrap();fire.animation18=3;fire.motion22=127;let wreck=state.objects[1].as_mut().unwrap();wreck.flag_delay1f=1;wreck.lifetime24=1;state.effect_world_mask4f=23;state.effect_wall_mask43=9;state.effect_world_suppression=17;}g.update_native_world_objects();let state=g.native_world.as_ref().unwrap();assert_eq!((state.effect_world_mask4f,state.effect_wall_mask43,state.effect_world_suppression),(0,0,0),"43f100/41f580 clear rather than restore caller masks");assert_eq!(state.objects[0].as_ref().unwrap().animation18,0);let wreck=state.objects[1].as_ref().unwrap();assert!(!wreck.active);assert_eq!(wreck.hp,237,"slot0 fire damage precedes slot1 forced expiry");assert_eq!((wreck.flag_delay1f,wreck.lifetime24),(1,1),"inactive later slots do not receive their callbacks");}
    #[test]fn ordinary_object_door_animation_stops_at_four_and_runs_only_on_the_shared_pulse(){let mut g=field();g.objects=vec![raw(31,255,0,400,400,0)];g.import_native_world(&Value::Null);g.set_native_object_animation(0,0,1);for tick in 0..8{g.motion_tick=tick;g.update_native_world_objects();assert_eq!(g.native_object_animation(0),Some((((tick/2)+1).min(4)as u8,if tick<6{1}else{0})));}assert!(g.clear_native_apc_object_contact(0));g.motion_tick=8;g.update_native_world_objects();assert_eq!(g.native_object_animation(0),Some((3,-1)));}
    #[test]fn native_bullet_and_blast_robot_deaths_use_distinct_real_pools(){
        let mut g=field();g.spawn("grunt",1,640.,640.,1,None);g.units[0].hp=0.;let id=g.units[0].id;let mut expected=g.original_rng.clone();expected.next_u32();
        g.apply_native_contact(Attack{target:id,damage:1,aim:0,facing:0,bullet:true});assert_eq!(g.units[0].hp,-1.);assert_eq!(g.units[0].native_slot,None);assert_eq!(g.native_deaths.records.iter().filter(|r|r.sprite!=-1).count(),1);assert!(g.native_world.as_ref().unwrap().effects.iter().all(Option::is_none));assert_eq!(g.original_rng.words,expected.words);assert!(g.effects.is_empty());
        let mut g=field();g.spawn("grunt",1,640.,640.,1,None);g.units[0].hp=0.;let id=g.units[0].id;let mut expected=g.original_rng.clone();for _ in 0..6{expected.next_u32();}g.apply_native_contact(Attack{target:id,damage:1,aim:0,facing:0,bullet:false});assert!(g.native_deaths.records.iter().all(|r|r.sprite==-1));let p=g.native_world.as_ref().unwrap().effects.iter().flatten().next().unwrap();assert_eq!((p.sprite,p.expiry,p.owner),(1903,0,1));assert_eq!(g.original_rng.words,expected.words);assert!(g.effects.is_empty());
    }
    #[test]fn destroyed_tank_allocates_native_wreck_terrain_and_removal_after_its_fragments(){
        let mut g=field();g.multiplayer=true;g.spawn("medium",1,640.,640.,1,None);g.units[0].hp=0.;let id=g.units[0].id;let mut expected=g.original_rng.clone();expected.multiplayer=true;for _ in 0..18{expected.next_u32();}
        g.apply_native_contact(Attack{target:id,damage:1,aim:0,facing:0,bullet:false});let state=g.native_world.as_ref().unwrap();let object=state.objects[0].as_ref().unwrap();assert_eq!((object.kind,object.hp,object.flags19,object.record.position),(6,1,0,[304,320]));assert_eq!(object.record.flags13,114);assert_eq!(object.unit,None);assert_eq!(g.objects[0]["type"],6);assert_eq!(g.objects[0]["nativeTeam"],1);assert_eq!(g.native_deaths.records.iter().filter(|r|r.sprite!=-1).count(),1);assert_eq!(g.original_rng.words,expected.words);for d in[0,1,128,129]{assert_eq!(g.world_code(20*128+19+d)&127,28);}assert_eq!(g.native_world.as_ref().unwrap().effects.iter().flatten().count(),5);
    }
    #[test]fn crew_killed_jeep_stays_usable_at_the_exact_58af0_origin_with_scan_mask_fifteen(){
        let mut g=field();g.spawn("jeep",1,640.,640.,1,None);g.units[0].hp=176.;g.import_native_world(&Value::Null);let before=g.units[0].clone();g.units[0].team=-1;g.units[0].crew=None;let mut expected=g.original_rng.clone();for _ in 0..6{expected.next_u32();}
        g.native_contact_removal(0,&before,super::super::original_death::RemovalSource::Bullet,true);g.sync_native_world();let object=g.native_world.as_ref().unwrap().objects[0].as_ref().unwrap();assert_eq!((object.kind,object.hp,object.flags19,object.record.position),(33,7,15,[316,313]));assert_eq!(object.unit,Some(before.id));assert_eq!((g.units[0].x,g.units[0].y,g.units[0].hp),(632.,626.,175.));assert_eq!(g.units[0].native_slot,None);assert!(g.native_deaths.records.iter().all(|r|r.sprite==-1));assert_eq!(g.original_rng.words,expected.words);
    }
    #[test]fn destroyed_hardware_expires_after_288_callbacks_and_restores_its_footprint(){
        let mut g=field();g.multiplayer=true;g.spawn("medium",1,640.,640.,1,None);g.units[0].hp=0.;let id=g.units[0].id;g.apply_native_contact(Attack{target:id,damage:1,aim:0,facing:0,bullet:false});for _ in 0..287{g.update_native_world_object_delays();}assert!(g.native_world.as_ref().unwrap().objects[0].as_ref().unwrap().active);g.update_native_world_object_delays();assert!(!g.native_world.as_ref().unwrap().objects[0].as_ref().unwrap().active);assert_eq!(g.objects[0]["nativeActive"],false);for d in[0,1,128,129]{assert_eq!(g.world_code(20*128+19+d)&127,0);}
    }

    #[test]
    fn object_contact_delay_matches_all_256_native_signed_byte_values(){
        let mut g=field();g.native_world=Some(State::empty());let object=g.make_object(28,[200,200],50).unwrap();g.native_world.as_mut().unwrap().objects[0]=Some(object);let mut hash=2166136261u32;
        for value in 0..256{g.set_native_flag_delay(0,value as u8);g.update_native_world_object_delays();let next=g.native_world_scan_objects()[0].flag_delay1f;for n in[value,u32::from(next)]{hash=(hash^n).wrapping_mul(16777619);}}
        assert_eq!(hash,2097262112,"ZNativeObjectDelayOracle");
        g.native_world.as_mut().unwrap().objects[0].as_mut().unwrap().record.position[0]=0;g.set_native_flag_delay(0,85);g.update_native_world_object_delays();assert_eq!(g.native_world_scan_objects()[0].flag_delay1f,85,"435d90 calls43f260 only for signedX>0");
    }

    #[test]
    fn neutral_object_binding_retires_before_its_pool_slot_is_reused(){
        let mut g=field();let id=neutral(&mut g,"medium",400.,400.,500.);g.objects=vec![raw(8,1,0,400,400,20)];g.import_native_world(&Value::Null);
        assert_eq!(g.native_object_slot(id),Some(0));assert_eq!(g.native_object_unit(0),Some(id));
        g.units.iter_mut().find(|u|u.id==id).unwrap().team=0;g.sync_native_world();assert_eq!(g.native_object_slot(id),None);assert_eq!(g.objects[0]["nativeActive"],false);
        assert_eq!(g.reserve_native_world_fire_slot(),Some(0));g.sync_native_world();assert_eq!(g.objects[0]["nativeActive"],false);assert_eq!(g.native_world.as_ref().unwrap().objects[0].as_ref().unwrap().fire,Some(0));
        g.release_native_world_fire_slot(0);assert_eq!(g.reserve_native_world_fire_slot(),Some(0));
    }
    #[test]
    fn one_object_hit_updates_its_neutral_unit_and_renderer_without_hitting_the_second(){
        let mut g=field();let a=neutral(&mut g,"medium",400.,400.,500.);let b=neutral(&mut g,"medium",400.,400.,500.);
        g.objects=vec![raw(8,1,0,400,400,20),raw(8,1,1,400,400,20)];g.import_native_world(&Value::Null);
        g.native_blast_environment(196,184,1,0,false);assert_eq!(g.units.iter().find(|u|u.id==a).unwrap().hp,475.);assert_eq!(g.units.iter().find(|u|u.id==b).unwrap().hp,500.);
        g.native_blast_environment(196,184,127,0,false);assert_eq!(g.units.iter().find(|u|u.id==a).unwrap().hp,-1.);assert_eq!(g.objects[0]["nativeActive"],false);assert_eq!(g.objects[1]["nativeActive"],true);
    }
    #[test]
    fn grenade_crate_secondary_blast_hits_buildings_and_suppresses_other_object_contacts(){
        let mut g=field();g.crates=vec![Crate{x:400.,y:400.,taken:false},Crate{x:420.,y:400.,taken:false}];g.objects=vec![raw(10,255,0,400,400,0),raw(10,255,1,420,400,0)];g.buildings.push(building(7,"robot",400.,400.,500.));g.import_native_world(&Value::Null);
        g.native_blast_environment(196,184,1,0,false);assert!(g.crates[0].taken);assert!(!g.crates[1].taken);assert_eq!(g.objects[0]["nativeActive"],false);assert_eq!(g.objects[1]["nativeActive"],true);assert_eq!(g.buildings[0].hp,372.);assert_eq!(g.blasts.len(),1);assert_eq!(g.blasts[0].strength,127);
    }
    #[test]
    fn native_build_pool_order_includes_friendly_factories(){
        let mut g=field();g.buildings.push(building(7,"robot",400.,400.,100.));g.buildings.push(building(8,"robot",400.,400.,100.));g.import_native_world(&Value::Null);
        g.native_blast_environment(200,200,10,0,false);assert_eq!(g.buildings[0].hp,90.);assert_eq!(g.buildings[1].hp,100.);
        g.native_blast_environment(184,184,10,0,true);assert_eq!(g.buildings[0].hp,90.);g.native_blast_environment(184,184,10,0,false);assert_eq!(g.buildings[0].hp,80.);
    }
    #[test]
    fn fort_work_proxy_and_global_health_are_independent_and_zero_does_not_win(){
        let mut g=field();let mut fort=building(7,"fort",800.,800.,1.);fort.team=1;fort.raw_base=Some((400.,400.));g.buildings.push(fort);
        g.import_native_world(&json!({"nativeWorldBuildings":[{"nativeSlot":4,"kind":2,"x":400,"y":416,"hp":1,"maxHp":1500,"buildingId":7,"fortProxy":true}]}));assert_eq!(g.native_world.as_ref().unwrap().buildings[4].record.hp,1);assert_eq!(g.native_work_health(&g.buildings[0]),(1.,1.));g.native_world.as_mut().unwrap().buildings[4].record.hp=1500;g.native_world.as_mut().unwrap().buildings[4].maximum=1500;
        g.native_blast_environment(400,400,1,0,false);assert_eq!(g.buildings[0].hp,0.);assert_eq!(g.winner,None);assert_eq!(g.native_work_health(&g.buildings[0]),(1499.,1500.));g.sync_native_fort_producer_health(7);assert_eq!(g.native_work_health(&g.buildings[0]),(0.,1.));
        g.native_blast_environment(400,400,1,0,false);assert_eq!(g.buildings[0].hp,-1.);assert_eq!(g.winner,Some(0));
    }
    #[test]
    fn fire_slots_share_the_real_two_hundred_object_capacity(){
        let mut g=field();g.objects=(0..200).map(|i|raw(4,255,i,64+(i%25)as i32*32,64+(i/25)as i32*32,0)).collect();g.import_native_world(&Value::Null);assert_eq!(g.reserve_native_world_fire_slot(),None);
        g.native_world.as_mut().unwrap().objects[77].as_mut().unwrap().active=false;assert_eq!(g.reserve_native_world_fire_slot(),Some(77));assert_eq!(g.objects[77]["nativeActive"],false);assert_eq!(g.reserve_native_world_fire_slot(),None);
    }
    #[test]
    fn bridge_collapse_uses_native_unit_classes_object_types_and_road_retirement(){
        let mut g=field();g.map.bridge_hp=vec![1.];g.map.bridge_max_hp=vec![600.];
        for y in 20..23{for x in 20..23{let j=y*g.map.width+x;g.map.bridge_base[j]=0;g.map.terrain_codes[j]=23;}}
        let bounds=[320,320,368,368];g.map.native_road_graph=Some(original_route::RoadGraph{regions:vec![original_route::RoadRegion{bounds,waypoint:0,next:vec![255]}]});
        g.spawn("grunt",0,660.,660.,1,None);let robot=g.units.last().unwrap().id;g.spawn("gatling",0,660.,660.,2,None);let gun=g.units.last().unwrap().id;let jeep=neutral(&mut g,"jeep",660.,660.,1000.);g.objects=vec![raw(33,13,0,660,660,40)];
        g.import_native_world(&json!({"bridges":[{"id":0,"nativeSlot":2,"nativeOrientation":0,"x":640,"y":640,"w":96,"h":96,"hp":1,"maxHp":600}]}));g.native_blast_environment(308,288,1,0,false);
        assert_eq!(g.map.bridge_hp[0],0.);assert_eq!(g.units.iter().find(|u|u.id==robot).unwrap().hp,-1.);assert!(g.units.iter().find(|u|u.id==gun).unwrap().hp>=0.);assert_eq!(g.units.iter().find(|u|u.id==jeep).unwrap().hp,-1.);assert_eq!(g.objects[0]["nativeActive"],false);assert_eq!(g.map.native_road_graph.as_ref().unwrap().regions[0].bounds,[0;4]);
        assert_eq!(g.world_code(21*128+21)&127,47);g.map.bridge_hp[0]=520.;g.sync_native_world();assert_eq!(g.native_world.as_ref().unwrap().bridges[0].record.flags15&2,2);
        g.map.bridge_hp[0]=600.;g.sync_native_world();assert_eq!(g.native_world.as_ref().unwrap().bridges[0].record.flags15&2,0);assert_eq!(g.map.native_road_graph.as_ref().unwrap().regions[0].bounds,bounds);
    }
    #[test]
    fn hardware_capture_clears_real_pool_restrictions_and_respects_level_eighteen(){
        let mut g=field();g.objects=vec![raw(8,1,0,400,400,20),raw(1,255,1,500,500,0)];g.import_native_world(&Value::Null);
        for o in g.native_world.as_mut().unwrap().objects.iter_mut().flatten(){o.flags26=1;}
        g.map.native_level_record=0x12;g.clear_native_hardware_restrictions(0);assert_eq!(g.native_world.as_ref().unwrap().objects[0].as_ref().unwrap().flags26,1);
        g.map.native_level_record=0;g.clear_native_hardware_restrictions(0);assert_eq!(g.native_world.as_ref().unwrap().objects[0].as_ref().unwrap().flags26,0);assert_eq!(g.native_world.as_ref().unwrap().objects[1].as_ref().unwrap().flags26,1);
    }
    #[test]
    fn original_object_particles_use_six_and_three_rng_words_and_the_separate_pool(){
        let mut g=field();g.objects=vec![raw(8,1,0,400,400,20)];g.import_native_world(&Value::Null);let mut expected=g.original_rng.clone();for _ in 0..16*6{expected.next_u32();}
        g.native_blast_environment(196,184,1,0,false);assert_eq!(g.original_rng.words,expected.words);assert_eq!(g.native_world.as_ref().unwrap().effects.iter().flatten().filter(|p|p.active).count(),16);
        g.update_native_world_effects();let draws=g.native_world_effects();assert_eq!(draws.len(),16);assert!(draws.iter().all(|d|d["x"]==412&&d["y"]==440));assert!(draws.iter().all(|d|(250..254).contains(&d["sprite"].as_i64().unwrap())));
        for _ in 0..16*6+16*3{expected.next_u32();}g.native_blast_environment(196,184,127,0,false);assert_eq!(g.original_rng.words,expected.words);assert_eq!(g.native_world.as_ref().unwrap().effects.iter().flatten().filter(|p|p.active).count(),48);
        g.native_world.as_mut().unwrap().effects.resize(1,None);let before=g.original_rng.words;g.spawn_native_object_particles(effects::Kind::Hit,[200,200]);assert_eq!(g.original_rng.words,before,"full particle pool does not consume RNG");
        g.native_world.as_mut().unwrap().effects[0].as_mut().unwrap().active=false;for _ in 0..6{expected.next_u32();}g.spawn_native_object_particles(effects::Kind::Hit,[200,200]);assert_eq!(g.original_rng.words,expected.words);
    }
    #[test]
    fn native_wall_wrapper_keeps_signed_byte_wrap_zero_hp_and_fire_suppression(){
        for(hp,strength,expected)in[(0,0,0.),(100,255,101.),(100,50,50.)]{let mut g=field();g.import_walls(&json!({"walls":[{"id":7,"x":640,"y":320,"w":32,"h":32,"hp":hp.max(1),"maxHp":hp.max(1),"nativeHp":hp}]})).unwrap();g.import_native_world(&Value::Null);
            g.native_blast_environment(308,128,strength,0,true);assert_eq!(g.walls[0].native_hp,Some(hp));
            g.native_blast_environment(308,128,strength,0,false);assert_eq!(g.walls[0].hp,expected);if expected==0.{assert!(!g.walls[0].alive());assert_eq!(g.map.terrain_codes[10*64+20],0);}
        }
    }
    fn expiring(g:&mut Game,kind:effects::Kind,point:[i16;2]){g.import_native_world(&Value::Null);let mut p=effects::initialize(kind,point,0,||0);p.lifetime=0;p.position[2]=-1;let s=g.native_world.as_mut().unwrap();s.last_projection=[0,0];s.effects[0]=Some(p);}
    #[test]
    fn crate_landing_has_one_forty_strength_contact_and_retains_its_residual_slot(){
        let mut g=field();g.buildings.push(building(7,"robot",480.,480.,500.));expiring(&mut g,effects::Kind::CrateBurst,[240,240]);g.update_native_world_effects();
        assert_eq!(g.buildings[0].hp,460.);assert_eq!(g.blasts.len(),1);assert_eq!(g.blasts[0].strength,40);let p=g.native_world.as_ref().unwrap().effects[0].as_ref().unwrap();assert_eq!((p.active,p.sprite,p.lifetime,p.expiry,p.position[2],p.maximum_phase),(true,473,21,0,-1,7));
        let before=g.events.len();g.update_native_world_effects();assert!(!g.events.iter().skip(before).any(|e|e["kind"]=="explosion"));assert_eq!(g.buildings[0].hp,460.);
    }
    #[test]
    fn falling_object_orders_zero_strength_direct_contact_before_one_delayed_blast(){
        let mut g=field();g.difficulty=0;g.spawn("grunt",0,472.,504.,1,None);let human=g.units.last().unwrap().id;let hp=g.units.last().unwrap().hp;g.spawn("grunt",1,472.,504.,2,None);let enemy=g.units.last().unwrap().id;g.maintain_native_group_rosters();g.buildings.push(building(7,"robot",480.,480.,500.));expiring(&mut g,effects::Kind::FallingObject(5960),[240,240]);g.update_native_world_effects();
        assert_eq!(g.units.iter().find(|u|u.id==human).unwrap().hp,hp-1.);assert_eq!(g.units.iter().find(|u|u.id==enemy).unwrap().hp,hp);assert_eq!(g.buildings[0].hp,500.);assert_eq!(g.blasts.len(),1);assert_eq!(g.blasts[0].strength,0);assert!(g.events.iter().any(|e|e["kind"]=="explosion"&&e["silent"]==true));assert!(!g.native_world.as_ref().unwrap().effects[0].as_ref().unwrap().active);
        for _ in 0..3{g.update_native_blasts();}assert_eq!(g.native_route_boundary,None,"fixture has real UNIT roster before blast admission");assert_eq!(g.units.iter().find(|u|u.id==human).unwrap().hp,hp-2.);assert_eq!(g.units.iter().find(|u|u.id==enemy).unwrap().hp,hp);
    }
    #[test]fn ambient_fragment_uses_expiry_one_strength_ten_and_shared_owner_palette(){
        let mut g=field();g.multiplayer=true;g.spawn("grunt",0,472.,504.,1,None);g.spawn("grunt",1,472.,504.,2,None);let hp=g.units.iter().map(|u|u.hp).collect::<Vec<_>>();g.buildings.push(building(7,"robot",480.,480.,500.));expiring(&mut g,effects::Kind::AmbientFragment(1234),[240,240]);g.update_native_world_effects();assert_eq!(g.buildings[0].hp,490.);for(i,u)in g.units.iter().enumerate(){assert_eq!(u.hp,hp[i]-10.);}assert!(g.blasts.is_empty());
        let mut g=field();g.import_native_world(&Value::Null);g.native_world.as_mut().unwrap().effect_owner=3;g.spawn_native_ambient_fragment([240,240],1234);let p=g.native_world.as_ref().unwrap().effects[0].as_ref().unwrap();assert_eq!((p.origin,p.sprite,p.expiry,p.owner),([80,148],1234,1,3));
    }
    #[test]fn hardware_fragment_land_contact_retains_residual_and_emits_two_eight_particle_bursts(){
        let mut g=field();g.buildings.push(building(7,"robot",480.,480.,500.));expiring(&mut g,effects::Kind::Fragment{sprite:1558,maximum:7},[240,240]);g.update_native_world_effects();assert_eq!(g.buildings[0].hp,373.);let state=g.native_world.as_ref().unwrap();assert_eq!(state.effects[0].as_ref().unwrap().sprite,473);assert_eq!(state.effects.iter().flatten().filter(|p|p.active).count(),17);assert_eq!(g.blasts.len(),1);assert_eq!(g.blasts[0].strength,127);
    }
    #[test]
    fn wet_falling_object_can_reuse_its_own_effect_slot_during_the_landing_callback(){
        let mut g=field();let id=neutral(&mut g,"medium",488.,512.,500.);g.objects=vec![raw(8,1,0,488,512,20)];g.map.terrain_codes[15*64+15]=9;expiring(&mut g,effects::Kind::FallingObject(5960),[240,240]);g.update_native_world_effects();
        let state=g.native_world.as_ref().unwrap();let p=state.effects[0].as_ref().unwrap();assert!(p.active);assert_eq!(p.sprite,250);assert_eq!(p.lifetime,200,"the native caller resumes the reused record without another lifetime decrement");assert_eq!(state.effects[1].as_ref().unwrap().lifetime,199);assert_eq!(g.units.iter().find(|u|u.id==id).unwrap().hp,500.);assert_eq!(g.blasts.len(),1);
    }
    #[test]
    fn full_effect_pool_consumes_only_the_explicit_burst_count_rng_word(){
        let mut g=field();expiring(&mut g,effects::Kind::CrateBurst,[240,240]);g.native_world.as_mut().unwrap().effects.truncate(1);let mut expected=g.original_rng.clone();expected.next_u32();g.update_native_world_effects();assert_eq!(g.original_rng.words,expected.words);assert_eq!(g.native_world.as_ref().unwrap().effects.len(),1);assert_eq!(g.native_world.as_ref().unwrap().effects[0].as_ref().unwrap().sprite,473);
    }
    #[test]
    fn destructible_object_and_grenade_crates_create_their_native_falling_profiles(){
        let mut g=field();let mut object=raw(0,255,0,480,480,0);object["templateFlags"]=json!(12);object["templateSprite"]=json!(5960);g.objects=vec![object];g.import_native_world(&Value::Null);g.native_blast_environment(236,224,1,0,false);let state=g.native_world.as_ref().unwrap();assert_eq!(state.effects.iter().flatten().filter(|p|p.active&&p.expiry==7).count(),1);let p=state.effects.iter().flatten().find(|p|p.expiry==7).unwrap();assert_eq!((p.sprite,p.origin,p.lifetime),(5960,[80,148],400));
        let mut g=field();g.crates=vec![Crate{x:400.,y:400.,taken:false}];g.objects=vec![raw(10,255,0,400,400,0)];g.import_native_world(&Value::Null);g.native_blast_environment(196,184,1,0,false);assert_eq!(g.native_world.as_ref().unwrap().effects.iter().flatten().filter(|p|p.expiry==6).count(),20);
    }
    #[test]
    fn native_flags_allocate_before_authored_objects_using_runtime_fort_membership(){
        let mut g=field();g.objects=vec![raw(10,255,0,400,400,0)];let mut fort=building(7,"fort",640.,640.,9999.);fort.raw_base=Some((320.,320.));fort.sector=1;g.buildings.push(fort);g.map.territory[20*64+20]=1;g.sectors.push(Sector{id:1,x:672.,y:672.,owner:1,home:true,counts_for_production:true});
        g.import_native_world(&json!({"sectors":[{"id":1,"nativeSlot":7,"nativeFlags":0,"rawFlagTile":[20,20]},{"id":0,"nativeSlot":2,"nativeFlags":131,"rawFlagTile":[10,10]}]}));
        let objects=g.native_world_scan_objects();assert_eq!(objects.len(),2);assert_eq!((objects[0].slot,objects[0].definition,objects[0].flags13,objects[0].flags19,objects[0].bucket12),(0,28,0,1,17));assert_eq!((objects[1].slot,objects[1].definition),(1,10));assert_eq!(g.objects[0]["nativeSlot"],1);assert_eq!(g.world_code(10*128+10)&127,35);
        g.set_native_object_scan_mask(0,!2);assert_eq!(g.native_world_scan_objects()[0].flags19,253);
    }
}

#[cfg(test)]mod terrain_presentation_tests{
 use super::*;
 #[test]fn object_occupancy_changes_native_collision_without_erasing_road_art(){let mut g=Game::new(0,1,19);g.map.original=true;let cell=2*128+2;let j=2*g.map.width+2;g.map.tiles[j]=1;g.map.terrain_codes.resize(g.map.tiles.len(),0);g.map.original_tiles.resize(g.map.tiles.len(),0);g.map.original_tiles[j]=77;g.set_world_code(cell,36);assert_eq!(g.world_code(cell),36);assert_eq!(g.map.tiles[j],1);assert_eq!(g.map.original_tiles[j]&255,77);g.set_world_code(cell,46);assert_eq!(g.world_code(cell),46);assert_eq!(g.map.tiles[j],1);g.set_world_code(cell,7);assert_eq!(g.map.tiles[j],1);g.set_world_code(cell,9);assert_eq!(g.map.tiles[j],3);}
}

impl Game{
 ///OwnGROUPFF+9 aliases OO24+21. None is an unmodeled byte, nevercold0.
 pub(crate) fn native_retired_group_ff_aa_byte(&self)->Option<i8>{let w=self.native_world.as_ref()?;w.objects[24].as_ref().map(|o|o.sprite_offset21).or(w.retired_preamble_offset21_24)}
 pub(crate) fn clear_native_retired_group_ff_aa(&mut self)->bool{let Some(old)=self.native_retired_group_ff_aa_byte()else{return false;};let next=(old as u8&0xdf)as i8;self.set_native_object_sprite_offset(24,next);if let Some(w)=&mut self.native_world{w.retired_preamble_offset21_24=Some(next);}true}
 #[cfg(test)]pub(crate) fn set_native_retired_group_ff_aa_fixture(&mut self,value:i8){if let Some(w)=&mut self.native_world{w.retired_preamble_offset21_24=Some(value);if let Some(o)=&mut w.objects[24]{o.sprite_offset21=value;}}}
}

#[cfg(test)]mod retired_group_alias_tests{
 use super::*;
 #[test]fn own_group_ff_aa_only_changes_actual_or_known_latent_object24_byte21(){let mut g=Game::new(0,1,1);g.map.original=true;g.units.clear();g.objects.clear();g.buildings.clear();g.crates.clear();g.sectors.clear();g.native_world=None;g.import_native_world(&Value::Null);assert_eq!(g.native_retired_group_ff_aa_byte(),None);assert!(!g.clear_native_retired_group_ff_aa());g.set_native_retired_group_ff_aa_fixture(-1);assert!(g.clear_native_retired_group_ff_aa());assert_eq!(g.native_retired_group_ff_aa_byte(),Some(-33));assert!(g.native_world.as_ref().unwrap().objects[24].is_none(),"AA cannot allocate or activate an OBJECT");g.import_native_world(&Value::Null);assert_eq!(g.native_retired_group_ff_aa_byte(),Some(-33),"460430 clearsX only; known warmbyte survives");let mut object=g.make_object(1,[400,500],3).unwrap();object.flags26=165;object.variant26=7;object.lifetime24=288;object.saved=[11,22,33,44];g.native_world.as_mut().unwrap().put_object(24,object);assert_eq!(g.native_retired_group_ff_aa_byte(),Some(0),"real43ecf0 constructor replaces byte21, not a warmalias merge");g.set_native_object_sprite_offset(24,-1);let before=g.native_world.as_ref().unwrap().objects[24].as_ref().unwrap().clone();assert!(g.clear_native_retired_group_ff_aa());let after=g.native_world.as_ref().unwrap().objects[24].as_ref().unwrap();assert_eq!(after.sprite_offset21,-33);assert_eq!((after.record.position,after.active,after.flags26,after.variant26,after.lifetime24,after.saved),(before.record.position,before.active,before.flags26,before.variant26,before.lifetime24,before.saved));assert!(g.native_world.as_ref().unwrap().objects[25].is_none(),"workshopOO25alias has a different owner");}
}
