use super::released_hardware;
use crate::{original_entry::HardwareAttributes,original_orders_game::{HardwarePoolSlot,release_hardware_slot}};
use serde_json::Value;

fn decode(value:&Value)->Vec<u8> {value.as_str().unwrap().as_bytes().chunks_exact(2)
    .map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}
fn pointer(bytes:&[u8],offset:usize)->u32{u32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap())}

#[test]
#[ignore = "requires private original fort hardware reentry trace"]
fn original_fort_reentry_uses_the_retired_physical_record_and_preserves_the_dummy() {
    let path=std::env::var("NATIVE_FORT_HARDWARE_TRACE").expect("private trace path");
    let text=std::fs::read_to_string(path).unwrap();
    let rows:Vec<Value>=text.lines().map(|s|serde_json::from_str(s).unwrap()).collect();
    assert_eq!(rows.len(),48);
    let mut copies=0;
    let mut nested=0;
    for (id,pair) in rows.chunks_exact(2).enumerate() {
        let (before,after)=(&pair[0],&pair[1]);
        assert_eq!(before["scenario"],id);assert_eq!(after["scenario"],id);
        assert_eq!(before["dummy"],after["dummy"],"shared dummy {id}");
        let original=decode(&before["units"]);
        let ending=decode(&after["units"]);
        let handle=pointer(&original,192+8);
        let objects=decode(&after["objects"]);
        let alive_object=i16::from_le_bytes(objects[..2].try_into().unwrap())!=-1;
        if !alive_object {
            assert_eq!(pointer(&ending,192+8),handle);
            continue;
        }
        let offset=usize::try_from(handle.checked_sub(0x59e6c2).unwrap()).unwrap();
        assert_eq!(offset%44,0);assert!(offset<240*44);
        let hardware=decode(&before["hardware"]);
        let mut typed=[HardwarePoolSlot{attributes:HardwareAttributes{bytes:hardware[offset..offset+44].try_into().unwrap()},..Default::default()}];
        release_hardware_slot(&mut typed,0);
        let retired=decode(&after["hardware"]);
        assert_eq!(typed[0].attributes.bytes.as_slice(),&retired[offset..offset+44],"warm W {id}");
        assert_eq!(pointer(&ending,192+8),u32::MAX);
        let kind=(pointer(&objects,4)-0x498f38)/26;
        let payload=released_hardware(typed[0].attributes,kind as u16);
        for field in [14,16,17,24,25,27,28,29,30] {
            assert_eq!(payload.attributes.byte(field),objects[field],"payload {id}/{field}");
        }
        assert_eq!((payload.hull,payload.maximum),(objects[15],objects[40]),"hull {id}");
        copies+=1;
        nested+=usize::from(after["visits"].as_array().unwrap().iter().any(|visit|visit[0]==0x41ddb0&&visit[1]==1));
    }
    assert_eq!((copies,nested),(10,5));
}
