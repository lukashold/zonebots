//! The ownerless, zero-strength 41f610 -> 420200 call. The ordinary projectile
//! caller is separate. Terrain words and classification lists are numeric
//! rules loaded by 42e1a0/41f7e0, rather than a browser terrain approximation.
#[path="original_fragment_blast_data.rs"] mod data;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum TerrainLeaf { None, Ground(i16), Floor(i16), Structure(i16) }
pub fn sample(point:[i16;2])->i16 {
    (point[1].wrapping_add(32) as u16&0xfff0).wrapping_mul(8)
        .wrapping_add((point[0].wrapping_add(12)>>4)as u16)as i16
}
pub fn admitted(point:[i16;2],world:[i16;2])->bool {
    point[1]>=-32&&point[0]<=world[0]&&point[1]<=world[1]
}
pub(crate) const SAMPLES:[i16;8]=[1,2,3,4,5,1,2,3];
pub fn sound(point:[i16;2],camera:[i16;2],size:[i16;2],gate:bool,quiet:bool,cycle:u8)->Option<i16>{
    let p=[point[0].wrapping_sub(camera[0]),point[1].wrapping_sub(camera[1])];
    (!gate&&!quiet&&p[0]>=-48&&p[1]>=-48&&i32::from(p[0])<=i32::from(size[0])+32&&i32::from(p[1])<=i32::from(size[1])+32)
        .then(||SAMPLES[usize::from(cycle&7)])
}
/// 420200 first probes the centre and four neighbours for bridge/wall contact.
/// When all miss, EDX still points SOUTH of the initial sample. Its subsequent
/// ground/floor search is south, southwest, southeast, centre-right, southeast.
pub fn terrain_leaf(base:i16,suppress_walls:bool,mut code:impl FnMut(i16)->u8)->(i16,TerrainLeaf){
    for d in [0i16,-1,1,-128,128]{let c=base.wrapping_add(d);let t=code(c)&127;
        if t==23||!suppress_walls&&matches!(t,1..=6|18){return(c,TerrainLeaf::Structure(c));}}
    for d in [128i16,127,129,1,129]{let c=base.wrapping_add(d);match code(c)&127{
        0=>return(c,if suppress_walls{TerrainLeaf::None}else{TerrainLeaf::Ground(c)}),
        7=>return(c,TerrainLeaf::Floor(c)),_=>{}}}
    (base.wrapping_add(129),TerrainLeaf::None)
}
pub fn damage_class(world:usize,tile:u8,terrain:u8)->i8{
    let index=u16::from(tile)+if terrain&128!=0{240}else{0};
    let mut class=-1;for(k,list)in data::CLASSES[world.min(4)].iter().enumerate(){if list.contains(&index){class=k as i8;}}class
}
pub fn words(world:usize)->&'static[u16;36]{&data::WORDS[world.min(4)]}
pub fn water_sprites(world:usize)->[i16;2]{[[1945,9],[2679,6],[3079,6],[3151,6],[3018,6]][world.min(4)]}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct WaterSeed{pub phase:i8,pub velocity:[i8;3],pub spin:i16}
///403100 and402d50 share these five RNG reads, including the inclusive0..40
///4609a0 range. Their expiry word is8 and5 respectively.
pub fn water_seed(mut random:impl FnMut()->u32)->WaterSeed{
    let phase=(random()&3)as i8;let x=((random()&65535)%41)as i8-20;let y=((random()&65535)%41)as i8-20;
    let z=-(((random()&15)as i8-3).max(3));let spin=(random()&63)as i16-32;WaterSeed{phase,velocity:[x,y,z],spin}
}
pub trait Terrain {
    fn tile(&self,cell:i16)->u8;
    fn terrain(&self,cell:i16)->u8;
    fn set(&mut self,cell:i16,word:u16);
    fn random(&mut self)->u32;
}
fn packed(e:&impl Terrain,c:i16)->u16{u16::from(e.tile(c))|(u16::from(e.terrain(c)&128)<<8)}
fn small(e:&impl Terrain,c:i16,w:&[u16])->bool{w[..12].contains(&packed(e,c))}
fn merge(e:&mut impl Terrain,origin:i16,class:i8,w:&[u16;36]){
    if class==2{return;}let choice=(e.random()&1)as usize;
    // Class-1 indexes the preceding four floor words or their duplicated copy.
    let index=if class==-1{16}else{20+(choice+class as usize*2)*4};
    for(d,n)in[0i16,1,128,129].into_iter().zip(0..4){e.set(origin.wrapping_add(d),w[index+n]);}
}
fn join(e:&mut impl Terrain,c:i16,class:i8,w:&[u16;36]){
    if !small(e,c,w){return;}let east=small(e,c.wrapping_add(1),w);
    if east&&small(e,c.wrapping_sub(127),w)&&small(e,c.wrapping_sub(128),w){merge(e,c.wrapping_sub(128),class,w);return;}
    if small(e,c.wrapping_add(128),w)&&small(e,c.wrapping_add(129),w)&&east{merge(e,c,class,w);}
    if small(e,c.wrapping_add(128),w)&&small(e,c.wrapping_add(127),w)&&small(e,c.wrapping_sub(1),w){merge(e,c.wrapping_sub(1),class,w);}
    if small(e,c.wrapping_sub(1),w)&&small(e,c.wrapping_sub(128),w)&&small(e,c.wrapping_sub(129),w){merge(e,c.wrapping_sub(129),class,w);}
}
/// 41fda0 / 420e20 / 41fc90. `previous_class` is 4a6f4b: the floor branch
/// deliberately retains it, whereas 41fb10 writes it for every ground call.
pub fn alter_terrain(world:usize,leaf:TerrainLeaf,previous_class:&mut i8,e:&mut impl Terrain){
    let w=words(world);match leaf{
        TerrainLeaf::Ground(c)=>{let class=damage_class(world,e.tile(c),e.terrain(c));*previous_class=class;
            if !w[..16].contains(&packed(e,c))&&(c as u16)<16384&&(0..2).contains(&class){let word=w[(e.random()&7)as usize+class as usize*8];e.set(c,word);}join(e,c,class,w);},
        TerrainLeaf::Floor(c)=>{let class=damage_class(world,e.tile(c),e.terrain(c));let already=e.terrain(c)&128!=0&&class==2&&w[16..20].contains(&packed(e,c));
            if !already&&class==2{let word=w[16+(e.random()&3)as usize];e.set(c,word);join(e,c,*previous_class,w);}},
        _=>{},
    }
}
#[cfg(test)]mod tests{
 use super::*;
 fn bytes_hash(b:impl IntoIterator<Item=u8>)->u32{b.into_iter().fold(2166136261,|h,v|(h^u32::from(v)).wrapping_mul(16777619))}
 struct Plane{tiles:Vec<u8>,codes:Vec<u8>,seed:u32,random:u32,draws:u32}
 impl Plane{fn index(&mut self,c:usize,index:u16){self.tiles[c]=if index>=240{(index-240)as u8}else{index as u8};self.codes[c]=if index>=240{128}else{0};}fn word(&mut self,c:usize,w:u16){self.tiles[c]=w as u8;self.codes[c]=(w>>8)as u8;}}
 impl Terrain for Plane{fn tile(&self,c:i16)->u8{self.tiles[c as usize]}fn terrain(&self,c:i16)->u8{self.codes[c as usize]}fn set(&mut self,c:i16,w:u16){self.word(c as usize,w);self.codes[c as usize]|=22;self.draws+=1;}fn random(&mut self)->u32{let r=self.seed*2027^self.random*7919^0x7731a5c9;self.random+=1;r}}
 #[test]fn both_water_child_constructors_match_512_executed_original_reused_records(){let mut hash=2166136261u32;for s in 0..512u32{let mut bytes:[u8;44]=std::array::from_fn(|i|(s*17+i as u32*29)as u8);let mut draws=0;let v=water_seed(||{let n=s*2027^draws*7919^0x7731a5c9;draws+=1;n});for(i,n)in v.velocity.into_iter().enumerate(){bytes[10+i]=n as u8;}bytes[13]=1;bytes[14]=3;bytes[18..20].copy_from_slice(&500i16.to_le_bytes());bytes[20..22].copy_from_slice(&(if s<256{8i16}else{5}).to_le_bytes());bytes[23]=v.phase as u8;bytes[28..30].copy_from_slice(&0i16.to_le_bytes());bytes[30..32].copy_from_slice(&v.spin.to_le_bytes());for n in[s,draws,bytes_hash(bytes)]{hash=(hash^n).wrapping_mul(16777619);}}assert_eq!(hash,1781977814,"original403100/402d50 preserve every unwritten byte");}
 #[test]fn actual420200_terrain_dispatch_keeps_register_driven_neighbour_order(){let oracle=std::env::var("Z_FRAGMENT_CHOICE_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|l|l.split(',').map(|n|n.parse::<i32>().unwrap()).collect::<Vec<_>>()).collect::<Vec<_>>());let mut all=2166136261u32;for s in 0..1024usize{let mut codes=vec![66;16384];for(i,d)in[0i16,-1,1,-128,128,127,129].into_iter().enumerate(){codes[(4256i16+d)as usize]=[0,7,9,1,23,18,5,66][((s>>(i%3*3))^(s/(i+1)))&7];}let(cell,leaf)=terrain_leaf(4256,s>=512,|c|codes[c as usize]);let(kind,called)=match leaf{TerrainLeaf::None=>(0,-999),TerrainLeaf::Ground(c)=>(1,i32::from(c)),TerrainLeaf::Floor(c)=>(2,i32::from(c)),TerrainLeaf::Structure(c)=>(3,i32::from(c))};let row=[s as i32,kind,called,i32::from(cell)];if let Some(o)=&oracle{assert_eq!(row.as_slice(),o[s].as_slice(),"420200 terrain selection{s}");}for n in row{all=(all^n as u32).wrapping_mul(16777619);}}assert_eq!(all,4253057555,"1024 original420200 calls with recorded world query and terrain leaves");}
 #[test]fn crater_constructor_and_1024_actual_native_ground_floor_merge_calls_match(){
    let oracle=std::env::var("Z_FRAGMENT_TERRAIN_ORACLE").ok().map(|p|std::fs::read_to_string(p).unwrap().lines().map(|l|serde_json::from_str::<serde_json::Value>(l).unwrap()).collect::<Vec<_>>());let mut all=2166136261u32;
    for s in 0..1024usize{let world=s%5;let w=words(world);let mut classes=[255u8;512];for(k,list)in data::CLASSES[world].iter().enumerate(){for &v in *list{classes[v as usize]=k as u8;}}
        let table=bytes_hash(classes.into_iter().chain(w[..20].iter().chain(&w[16..20]).chain(&w[20..]).flat_map(|n|n.to_le_bytes())));
        let mut e=Plane{tiles:vec![0;16384],codes:vec![0;16384],seed:s as u32,random:0,draws:0};let category=s/5%4;let first=(category<3).then(||classes.iter().position(|&v|v==category as u8)).flatten();if let Some(index)=first{e.index(8256,index as u16);}else{e.word(8256,w[s%12]);}
        let squares=[[1i16,-127,-128],[128,129,1],[128,127,-1],[-1,-128,-129]];let mask=s/80%16;for(k,ds)in squares.into_iter().enumerate(){if mask&(1<<k)!=0{for d in ds{e.word((8256i16+d)as usize,w[s%12]);}}}
        let mut class=[-1,0,1,2][s/20%4];alter_terrain(world,if s<512{TerrainLeaf::Ground(8256)}else{TerrainLeaf::Floor(8256)},&mut class,&mut e);
        let row=[s as u32,table,e.random,e.draws,class as u32,bytes_hash(e.tiles),bytes_hash(e.codes)];if let Some(o)=&oracle{let n=&o[s];let expected=[n["scenario"].as_u64().unwrap()as u32,n["tables"].as_u64().unwrap()as u32,n["random"].as_u64().unwrap()as u32,n["draws"].as_u64().unwrap()as u32,n["class"].as_i64().unwrap()as u32,n["tiles"].as_u64().unwrap()as u32,n["terrain"].as_u64().unwrap()as u32];assert_eq!(row,expected,"native terrain scenario{s}");}for n in row{all=(all^n).wrapping_mul(16777619);}
    }assert_eq!(all,2002137334,"42e1a0 +41f7e0 and real41fda0/420e20/41fc90/41fbd0, recorded RNG/redraw leaves");
 }
}
