use rltk::rex::XpFile;

// TODO(DP): use enum for constants
rltk::embedded_resource!(SMALL_DUNGEON, "../resources/SmallDungeon_80x50.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE2, "../resources/wfc-demo2.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(SMALL_DUNGEON, "../resources/SmallDungeon_80x50.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE2, "../resources/wfc-demo2.xp");

        RexAssets {
            menu: XpFile::from_resource("../resources/SmallDungeon_80x50.xp").unwrap(),
        }
    }
}
