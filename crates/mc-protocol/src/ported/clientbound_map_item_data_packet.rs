#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapId(pub i32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapDecoration {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapPatch {
    pub data: Vec<u8>,
}

pub trait MapItemSavedDataLike {
    fn add_client_side_decorations(&mut self, decorations: &[MapDecoration]);
    fn apply_color_patch(&mut self, patch: &MapPatch);
}

pub trait ClientGamePacketListener {
    fn handle_map_item_data(&mut self, packet: &ClientboundMapItemDataPacket);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundMapItemDataPacket {
    map_id: MapId,
    scale: i8,
    locked: bool,
    decorations: Option<Vec<MapDecoration>>,
    color_patch: Option<MapPatch>,
}

impl ClientboundMapItemDataPacket {
    // JAVA: net.minecraft.network.protocol.game.ClientboundMapItemDataPacket#<init>
    pub fn new(
        map_id: MapId,
        scale: i8,
        locked: bool,
        decorations: Option<Vec<MapDecoration>>,
        color_patch: Option<MapPatch>,
    ) -> Self {
        Self {
            map_id,
            scale,
            locked,
            decorations,
            color_patch,
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMapItemDataPacket#<init>
    pub fn from_nullable(
        map_id: MapId,
        scale: i8,
        locked: bool,
        decorations: Option<Vec<MapDecoration>>,
        color_patch: Option<MapPatch>,
    ) -> Self {
        Self::new(
            map_id,
            scale,
            locked,
            decorations.map(|d| d.into_iter().collect()),
            color_patch,
        )
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMapItemDataPacket#mapId
    pub fn map_id(&self) -> MapId {
        self.map_id
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMapItemDataPacket#scale
    pub fn scale(&self) -> i8 {
        self.scale
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMapItemDataPacket#locked
    pub fn locked(&self) -> bool {
        self.locked
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMapItemDataPacket#decorations
    pub fn decorations(&self) -> Option<&[MapDecoration]> {
        self.decorations.as_deref()
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMapItemDataPacket#colorPatch
    pub fn color_patch(&self) -> Option<&MapPatch> {
        self.color_patch.as_ref()
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMapItemDataPacket#applyToMap
    pub fn apply_to_map<M: MapItemSavedDataLike + ?Sized>(&self, map: &mut M) {
        if let Some(decorations) = &self.decorations {
            map.add_client_side_decorations(decorations);
        }
        if let Some(patch) = &self.color_patch {
            map.apply_color_patch(patch);
        }
    }

    // JAVA: net.minecraft.network.protocol.game.ClientboundMapItemDataPacket#handle
    pub fn handle<L: ClientGamePacketListener + ?Sized>(&self, listener: &mut L) {
        listener.handle_map_item_data(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockMap {
        decorations: Vec<MapDecoration>,
        patch_applied: Option<MapPatch>,
        call_order: Vec<&'static str>,
    }

    impl MapItemSavedDataLike for MockMap {
        fn add_client_side_decorations(&mut self, decorations: &[MapDecoration]) {
            self.call_order.push("decorations");
            self.decorations = decorations.to_vec();
        }

        fn apply_color_patch(&mut self, patch: &MapPatch) {
            self.call_order.push("patch");
            self.patch_applied = Some(patch.clone());
        }
    }

    #[test]
    fn from_nullable_copies_and_wraps() {
        let decos = vec![MapDecoration { data: vec![1, 2] }];
        let patch = MapPatch { data: vec![9] };
        let pkt = ClientboundMapItemDataPacket::from_nullable(
            MapId(7),
            3i8,
            true,
            Some(decos.clone()),
            Some(patch.clone()),
        );
        assert_eq!(pkt.map_id(), MapId(7));
        assert_eq!(pkt.scale(), 3i8);
        assert!(pkt.locked());
        assert_eq!(pkt.decorations(), Some(decos.as_slice()));
        assert_eq!(pkt.color_patch(), Some(&patch));
    }

    #[test]
    fn from_nullable_none_fields() {
        let pkt = ClientboundMapItemDataPacket::from_nullable(MapId(0), -1i8, false, None, None);
        assert_eq!(pkt.map_id(), MapId(0));
        assert_eq!(pkt.scale(), -1i8);
        assert!(!pkt.locked());
        assert!(pkt.decorations().is_none());
        assert!(pkt.color_patch().is_none());
    }

    #[test]
    fn apply_to_map_delegates_in_order() {
        let pkt = ClientboundMapItemDataPacket::new(
            MapId(1),
            0,
            false,
            Some(vec![MapDecoration { data: vec![5] }]),
            Some(MapPatch { data: vec![6] }),
        );
        let mut map = MockMap::default();
        pkt.apply_to_map(&mut map);
        assert_eq!(map.call_order, vec!["decorations", "patch"]);
        assert_eq!(map.decorations.len(), 1);
        assert_eq!(map.patch_applied.unwrap().data, vec![6]);
    }

    #[test]
    fn apply_to_map_skips_absent_parts() {
        let pkt = ClientboundMapItemDataPacket::new(MapId(2), 0, false, None, None);
        let mut map = MockMap::default();
        pkt.apply_to_map(&mut map);
        assert!(map.call_order.is_empty());
        assert!(map.decorations.is_empty());
        assert!(map.patch_applied.is_none());
    }

    #[test]
    fn handle_delegates_to_listener() {
        struct MockListener {
            seen_id: Option<i32>,
        }
        impl ClientGamePacketListener for MockListener {
            fn handle_map_item_data(&mut self, packet: &ClientboundMapItemDataPacket) {
                self.seen_id = Some(packet.map_id().0);
            }
        }

        let pkt = ClientboundMapItemDataPacket::new(MapId(99), 1, true, None, None);
        let mut listener = MockListener { seen_id: None };
        pkt.handle(&mut listener);
        assert_eq!(listener.seen_id, Some(99));
    }
}
