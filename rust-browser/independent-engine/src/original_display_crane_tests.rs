use super::*;
fn mix(h:&mut u32,values:impl IntoIterator<Item=u32>){for n in values{*h=(*h^n).wrapping_mul(16777619);}}
fn hash(bytes:impl IntoIterator<Item=u8>)->u32{let mut h=2166136261;mix(&mut h,bytes.into_iter().map(u32::from));h}
fn hex(b:&[u8])->String{b.iter().map(|n|format!("{n:02x}")).collect()}
#[test]fn crane_display_all2048_actual_native_compositions_clips_warm_bytes_and_raster_arguments(){
 let trace=std::env::var_os("Z_NATIVE_TRACE_DIR").and_then(|d|std::fs::read_to_string(std::path::PathBuf::from(d).join("V45-crane-display.jsonl")).ok());
 let rows=trace.as_ref().map(|s|s.lines().map(|l|serde_json::from_str::<serde_json::Value>(l).unwrap()).collect::<Vec<_>>());
 let mut golden=2166136261;let(mut visible,mut upper,mut lower,mut negative)=(0,0,0,0);
 for s in 0..2048usize{
  let heading=(s%8)as u8;let turret=(s/8%8)as i8;let frame=[0,1,2][s/64%3];let work=[-2,-1,0,1,6,7,65,70,71,127][s/192%10];
  let terrain=[0,56,58,47,48,50,57,30,62][s/7%9];let mode=s/9%12;let kind=[-1,0,0,0,1,4,2,-1,-1,-1,4,1][mode];let by=[536,568,600,601][s/5%4];let hp=if mode==2{1}else if mode==3{-1}else{0};
  let elevation=[-32,0,7][s/11%3];let flags64=[0,1,2,3,4,5,6,7,16,17,20,21][s/2%12];let mask98=[0,1,3,7][s/5%4];let allow=s/17%2!=0;
  let camera=[[0,0],[40,60],[480,580],[531,0],[532,0],[0,647],[0,648],[600,700]][s/29%8];let screen=[500i16.wrapping_sub(camera[0]),600i16.wrapping_sub(camera[1])];
  let admitted=screen[0]>=-31&&screen[0]<1024&&screen[1]>=-47&&screen[1]<1024;
  let mut builds=vec![Building{position:[0,0],kind:-1,hp:0};35];if kind>=0{builds[if s%2==0{0}else{17}]=Building{position:[480,by],kind,hp};}
  if mode==10{builds[0]=Building{position:[480,580],kind:0,hp:1};builds[34]=Building{position:[480,568],kind:4,hp:0};}
  if mode==11{builds[0]=Building{position:[480,568],kind:1,hp:0};builds[34]=Building{position:[480,568],kind:4,hp:0};}
  let forts=[if mode==7{[500,600]}else{[2000,2000]},if mode==8{[500,600]}else{[3000,3000]}];
  let clip=if admitted{select_clip(terrain,[500,600],&builds,Some(forts),allow).unwrap()}else{None};let line=clip.map_or(123,|c|c.line);let clipmode=clip.map_or(0,|c|c.mode);upper+=usize::from(clipmode==4);lower+=usize::from(clipmode==8);
  let(mut header,mut extra,mut raster)=(Vec::new(),Vec::new(),Vec::<[i32;9]>::new());let(mut fa,mut ma)=(flags64,mask98);
  if admitted{visible+=1;let out=draw(Input{position:screen,heading,frame,turret,work,flags64,mask98,elevation},clip,0).unwrap();header=out.header.bytes().unwrap().to_vec();let previous=(0..3).map(|n|std::array::from_fn(|i|(s*17+(n*16+i)*23)as u8)).collect::<Vec<[u8;16]>>();extra=packed_layers(&out,&previous,123);fa=out.flags64;ma=out.mask98;
   for l in out.layers{let r=l.rectangle(camera[1]).unwrap_or([0,0,-1,0]);negative+=usize::from(r[2]>=0&&r[3]<0);let p=if l.palette<=3{l.palette}else{255};raster.push([if s/13%2==0{0x466090}else{0x466180},i32::from(l.position[0]),i32::from(l.position[1]),i32::from(l.sprite),i32::from(p),r[0],r[1],r[2],r[3]]);}
  }
  mix(&mut golden,[s as u32,u32::from(clipmode),line as u32,u32::from(fa),u32::from(ma),header.len()as u32,hash(header.iter().copied()),extra.len()as u32,hash(extra.iter().copied()),raster.len()as u32]);for t in &raster{mix(&mut golden,t.map(|n|n as u32));}
  if let Some(rows)=&rows{let r=&rows[s];assert_eq!(r["layers"],hex(&extra),"warm layer{s}");if admitted{let q=r["queueBeforeSort"].as_str().unwrap();assert_eq!(&q[q.len()-16..],hex(&header),"header{s}");}
   assert_eq!((r["clip"].as_u64().unwrap(),r["line"].as_i64().unwrap(),r["flagsAfter"].as_u64().unwrap(),r["maskAfter"].as_u64().unwrap()),(u64::from(clipmode),i64::from(line),u64::from(fa),u64::from(ma)),"state{s}");
   let native=r["trace"].as_array().unwrap().iter().filter(|t|t[3].as_i64().unwrap()<5900).cloned().collect::<Vec<_>>();assert_eq!(serde_json::json!(raster),serde_json::json!(native),"raster{s}");
  }
 }
 assert_eq!((visible,upper,lower,negative),(1305,517,255,85));assert_eq!(golden,10607035,"2048 actual55f10/455820 executions; private full19-field validator FNV1704816281; GPU endpoints remain recorded boundaries");
}
#[test]fn crane_display_ruins_precede_an_earlier_live_record_and_type2_ruins_ignore_live_override(){
 let mut bs=vec![Building{position:[0,0],kind:-1,hp:0};35];bs[0]=Building{position:[480,580],kind:0,hp:1};bs[34]=Building{position:[480,568],kind:4,hp:0};assert_eq!(select_clip(56,[500,600],&bs,None,false),Ok(Some(Clip{mode:4,line:635})));
 bs[34]=Building{position:[480,600],kind:2,hp:0};assert_eq!(select_clip(58,[500,600],&bs,None,false),Ok(Some(Clip{mode:8,line:599})));bs[34].hp=1;assert_eq!(select_clip(58,[500,600],&bs,None,false),Ok(Some(Clip{mode:4,line:630})));
}
#[test]fn crane_display_signed9a_and_w21_are_not_clamped_or_mapped_from_body_heading(){
 let mut i=Input{position:[100,100],heading:3,frame:2,turret:7,work:71,flags64:5,mask98:3,elevation:0};let d=draw(i,None,17).unwrap();assert_eq!(d.layers.iter().map(|l|l.sprite).collect::<Vec<_>>(),[5833,5805,5825]);assert_eq!(d.header.sprite,17);assert_eq!((d.flags64,d.mask98),(1,7));assert!(d.layers.iter().all(|l|l.palette==1));
 i.work=-2;assert_eq!(draw(i,None,0).unwrap().layers[0].sprite,5826);i.turret=-1;assert!(matches!(draw(i,None,0),Err(Boundary::InvalidW21(-1))));i.turret=8;assert!(matches!(draw(i,None,0),Err(Boundary::InvalidW21(8))));
}
