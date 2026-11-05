use super::*;

#[test]
fn fort_retirement_changes_the_existing_visibility_and_mission_activity_without_hiding_other_fort() {
    let mut game=Game::new(0,1,72);
    game.map.original=true;
    game.map.width=64;game.map.height=64;
    game.map.tiles=vec![0;4096];game.map.terrain_codes=vec![0;4096];game.map.native_terrain_plane=vec![0;16384];
    game.map.bridge_base=vec![-1;4096];game.map.original_tiles=vec![0;4096];
    game.units.clear();game.objects.clear();game.crates.clear();game.sectors.clear();game.buildings.retain(|b|b.kind=="fort");
    for (i,b) in game.buildings.iter_mut().enumerate(){
        b.raw_base=Some((320.,160.+i as f64*400.));
        b.x=640.;b.y=320.+i as f64*800.;
    }
    game.native_world=None;
    game.import_native_world(&Value::Null);
    game.native_feedback.scene_gate=1;
    game.native_view=Some(([0,0],[1024,1024]));
    let visible=|game:&Game|game.project_native_flag_display().iter()
        .filter(|(_,row)|row["source"]=="fortBanner").map(|(_,row)|row["nativeSlot"].as_u64().unwrap()).collect::<Vec<_>>();
    assert_eq!(game.native_fort_team_activity(),Some([255,255,0,0]));
    assert_eq!(visible(&game),[0,1]);
    assert_eq!(game.retire_native_fort_team(2),Some(Ok(false)));
    assert_eq!(game.native_fort_team_activity(),Some([255,0,0,0]));
    assert_eq!(visible(&game),[0]);
    let end=game.native_world.as_ref().unwrap().fort_end;
    assert_eq!((end.scene_dirty,end.defeated_owner,end.palette_row,end.palette_cursor),(1,2,20,255));
    game.import_native_world(&Value::Null);
    game.native_display.clear_for_mission();
    assert_eq!(game.native_fort_team_activity(),Some([255,255,0,0]));
    assert_eq!(visible(&game),[0,1]);
    let end=game.native_world.as_ref().unwrap().fort_end;
    assert_eq!((end.scene_dirty,end.defeated_owner,end.palette_row,end.palette_cursor),(0,2,-1,255));
}
