use std::fmt;

/// Error type for game protocol codec operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketCodecError {
    SkipDecoder(String),
    SkipEncoder(String),
}

impl fmt::Display for PacketCodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SkipDecoder(msg) => write!(f, "SkipPacketDecoderException: {msg}"),
            Self::SkipEncoder(msg) => write!(f, "SkipPacketEncoderException: {msg}"),
        }
    }
}

impl std::error::Error for PacketCodecError {}

/// Minimal stream codec abstraction mirroring `net.minecraft.network.codec.StreamCodec`.
pub trait StreamCodec<B, V> {
    fn decode(&self, buf: &mut B) -> Result<V, PacketCodecError>;
    fn encode(&self, buf: &mut B, value: &V) -> Result<(), PacketCodecError>;
}

// JAVA: net.minecraft.network.protocol.game.GameProtocols$Context
pub trait Context {
    // JAVA: net.minecraft.network.protocol.game.GameProtocols$Context#hasInfiniteMaterials
    fn has_infinite_materials(&self) -> bool;
}

/// Codec wrapper that rejects encode/decode when the context lacks infinite materials.
#[derive(Debug, Clone)]
pub struct HasInfiniteMaterialsCodec<C> {
    inner: C,
    has_infinite_materials: bool,
}

// JAVA: net.minecraft.network.protocol.game.GameProtocols#HAS_INFINITE_MATERIALS
pub fn has_infinite_materials_codec<B, V, C, Ctx>(
    inner: C,
    ctx: &Ctx,
) -> HasInfiniteMaterialsCodec<C>
where
    C: StreamCodec<B, V>,
    Ctx: Context,
{
    HasInfiniteMaterialsCodec {
        inner,
        has_infinite_materials: ctx.has_infinite_materials(),
    }
}

impl<B, V, C> StreamCodec<B, V> for HasInfiniteMaterialsCodec<C>
where
    C: StreamCodec<B, V>,
{
    // JAVA: net.minecraft.network.protocol.game.GameProtocols#decode
    fn decode(&self, buf: &mut B) -> Result<V, PacketCodecError> {
        if !self.has_infinite_materials {
            return Err(PacketCodecError::SkipDecoder(
                "Not in creative mode".to_string(),
            ));
        }
        self.inner.decode(buf)
    }

    // JAVA: net.minecraft.network.protocol.game.GameProtocols#encode
    fn encode(&self, buf: &mut B, value: &V) -> Result<(), PacketCodecError> {
        if !self.has_infinite_materials {
            return Err(PacketCodecError::SkipEncoder(
                "Not in creative mode".to_string(),
            ));
        }
        self.inner.encode(buf, value)
    }
}

/// Connection protocol identifier used by game protocol templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionProtocol {
    Play,
}

/// A single packet registration entry in a protocol template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketRegistration {
    pub packet_type: &'static str,
    pub stream_codec: &'static str,
    pub modifier: Option<&'static str>,
}

/// Bundle packet registration for clientbound protocol templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleRegistration {
    pub packet_type: &'static str,
    pub bundle_factory: &'static str,
    pub delimiter: &'static str,
}

/// Serverbound game protocol template data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundTemplate {
    pub protocol: ConnectionProtocol,
    pub packets: Vec<PacketRegistration>,
}

/// Clientbound game protocol template data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTemplate {
    pub protocol: ConnectionProtocol,
    pub bundle: BundleRegistration,
    pub packets: Vec<PacketRegistration>,
}

fn reg(packet_type: &'static str, stream_codec: &'static str) -> PacketRegistration {
    PacketRegistration {
        packet_type,
        stream_codec,
        modifier: None,
    }
}

fn reg_mod(
    packet_type: &'static str,
    stream_codec: &'static str,
    modifier: &'static str,
) -> PacketRegistration {
    PacketRegistration {
        packet_type,
        stream_codec,
        modifier: Some(modifier),
    }
}

// JAVA: net.minecraft.network.protocol.game.GameProtocols#SERVERBOUND_TEMPLATE
pub fn serverbound_template() -> ServerboundTemplate {
    ServerboundTemplate {
        protocol: ConnectionProtocol::Play,
        packets: vec![
            reg("GamePacketTypes.SERVERBOUND_ACCEPT_TELEPORTATION", "ServerboundAcceptTeleportationPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_BLOCK_ENTITY_TAG_QUERY", "ServerboundBlockEntityTagQueryPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_BUNDLE_ITEM_SELECTED", "ServerboundSelectBundleItemPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CHANGE_DIFFICULTY", "ServerboundChangeDifficultyPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CHANGE_GAME_MODE", "ServerboundChangeGameModePacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CHAT_ACK", "ServerboundChatAckPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CHAT_COMMAND", "ServerboundChatCommandPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CHAT_COMMAND_SIGNED", "ServerboundChatCommandSignedPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CHAT", "ServerboundChatPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CHAT_SESSION_UPDATE", "ServerboundChatSessionUpdatePacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CHUNK_BATCH_RECEIVED", "ServerboundChunkBatchReceivedPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CLIENT_COMMAND", "ServerboundClientCommandPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CLIENT_TICK_END", "ServerboundClientTickEndPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.SERVERBOUND_CLIENT_INFORMATION", "ServerboundClientInformationPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_COMMAND_SUGGESTION", "ServerboundCommandSuggestionPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CONFIGURATION_ACKNOWLEDGED", "ServerboundConfigurationAcknowledgedPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CONTAINER_BUTTON_CLICK", "ServerboundContainerButtonClickPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CONTAINER_CLICK", "ServerboundContainerClickPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CONTAINER_CLOSE", "ServerboundContainerClosePacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED", "ServerboundContainerSlotStateChangedPacket.STREAM_CODEC"),
            reg("CookiePacketTypes.SERVERBOUND_COOKIE_RESPONSE", "ServerboundCookieResponsePacket.STREAM_CODEC"),
            reg("CommonPacketTypes.SERVERBOUND_CUSTOM_PAYLOAD", "ServerboundCustomPayloadPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST", "ServerboundDebugSubscriptionRequestPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_EDIT_BOOK", "ServerboundEditBookPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_ENTITY_TAG_QUERY", "ServerboundEntityTagQueryPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_INTERACT", "ServerboundInteractPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_JIGSAW_GENERATE", "ServerboundJigsawGeneratePacket.STREAM_CODEC"),
            reg("CommonPacketTypes.SERVERBOUND_KEEP_ALIVE", "ServerboundKeepAlivePacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_LOCK_DIFFICULTY", "ServerboundLockDifficultyPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_MOVE_PLAYER_POS", "ServerboundMovePlayerPacket.Pos.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_MOVE_PLAYER_POS_ROT", "ServerboundMovePlayerPacket.PosRot.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_MOVE_PLAYER_ROT", "ServerboundMovePlayerPacket.Rot.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_MOVE_PLAYER_STATUS_ONLY", "ServerboundMovePlayerPacket.StatusOnly.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_MOVE_VEHICLE", "ServerboundMoveVehiclePacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_PADDLE_BOAT", "ServerboundPaddleBoatPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_PICK_ITEM_FROM_BLOCK", "ServerboundPickItemFromBlockPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_PICK_ITEM_FROM_ENTITY", "ServerboundPickItemFromEntityPacket.STREAM_CODEC"),
            reg("PingPacketTypes.SERVERBOUND_PING_REQUEST", "ServerboundPingRequestPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_PLACE_RECIPE", "ServerboundPlaceRecipePacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_PLAYER_ABILITIES", "ServerboundPlayerAbilitiesPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_PLAYER_ACTION", "ServerboundPlayerActionPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_PLAYER_COMMAND", "ServerboundPlayerCommandPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_PLAYER_INPUT", "ServerboundPlayerInputPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_PLAYER_LOADED", "ServerboundPlayerLoadedPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.SERVERBOUND_PONG", "ServerboundPongPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS", "ServerboundRecipeBookChangeSettingsPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE", "ServerboundRecipeBookSeenRecipePacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_RENAME_ITEM", "ServerboundRenameItemPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.SERVERBOUND_RESOURCE_PACK", "ServerboundResourcePackPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SEEN_ADVANCEMENTS", "ServerboundSeenAdvancementsPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SELECT_TRADE", "ServerboundSelectTradePacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SET_BEACON", "ServerboundSetBeaconPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SET_CARRIED_ITEM", "ServerboundSetCarriedItemPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SET_COMMAND_BLOCK", "ServerboundSetCommandBlockPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SET_COMMAND_MINECART", "ServerboundSetCommandMinecartPacket.STREAM_CODEC"),
            reg_mod("GamePacketTypes.SERVERBOUND_SET_CREATIVE_MODE_SLOT", "ServerboundSetCreativeModeSlotPacket.STREAM_CODEC", "HAS_INFINITE_MATERIALS"),
            reg("GamePacketTypes.SERVERBOUND_SET_JIGSAW_BLOCK", "ServerboundSetJigsawBlockPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SET_STRUCTURE_BLOCK", "ServerboundSetStructureBlockPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SET_TEST_BLOCK", "ServerboundSetTestBlockPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SIGN_UPDATE", "ServerboundSignUpdatePacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_SWING", "ServerboundSwingPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_TELEPORT_TO_ENTITY", "ServerboundTeleportToEntityPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION", "ServerboundTestInstanceBlockActionPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_USE_ITEM_ON", "ServerboundUseItemOnPacket.STREAM_CODEC"),
            reg("GamePacketTypes.SERVERBOUND_USE_ITEM", "ServerboundUseItemPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.SERVERBOUND_CUSTOM_CLICK_ACTION", "ServerboundCustomClickActionPacket.STREAM_CODEC"),
        ],
    }
}

// JAVA: net.minecraft.network.protocol.game.GameProtocols#CLIENTBOUND_TEMPLATE
pub fn clientbound_template() -> ClientboundTemplate {
    ClientboundTemplate {
        protocol: ConnectionProtocol::Play,
        bundle: BundleRegistration {
            packet_type: "GamePacketTypes.CLIENTBOUND_BUNDLE",
            bundle_factory: "ClientboundBundlePacket::new",
            delimiter: "ClientboundBundleDelimiterPacket",
        },
        packets: vec![
            reg("GamePacketTypes.CLIENTBOUND_ADD_ENTITY", "ClientboundAddEntityPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_ANIMATE", "ClientboundAnimatePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_AWARD_STATS", "ClientboundAwardStatsPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_BLOCK_CHANGED_ACK", "ClientboundBlockChangedAckPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_BLOCK_DESTRUCTION", "ClientboundBlockDestructionPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_BLOCK_ENTITY_DATA", "ClientboundBlockEntityDataPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_BLOCK_EVENT", "ClientboundBlockEventPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_BLOCK_UPDATE", "ClientboundBlockUpdatePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_BOSS_EVENT", "ClientboundBossEventPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CHANGE_DIFFICULTY", "ClientboundChangeDifficultyPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CHUNK_BATCH_FINISHED", "ClientboundChunkBatchFinishedPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CHUNK_BATCH_START", "ClientboundChunkBatchStartPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CHUNKS_BIOMES", "ClientboundChunksBiomesPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CLEAR_TITLES", "ClientboundClearTitlesPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_COMMAND_SUGGESTIONS", "ClientboundCommandSuggestionsPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_COMMANDS", "ClientboundCommandsPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CONTAINER_CLOSE", "ClientboundContainerClosePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CONTAINER_SET_CONTENT", "ClientboundContainerSetContentPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CONTAINER_SET_DATA", "ClientboundContainerSetDataPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CONTAINER_SET_SLOT", "ClientboundContainerSetSlotPacket.STREAM_CODEC"),
            reg("CookiePacketTypes.CLIENTBOUND_COOKIE_REQUEST", "ClientboundCookieRequestPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_COOLDOWN", "ClientboundCooldownPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS", "ClientboundCustomChatCompletionsPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_CUSTOM_PAYLOAD", "ClientboundCustomPayloadPacket.GAMEPLAY_STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_DAMAGE_EVENT", "ClientboundDamageEventPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_DEBUG_BLOCK_VALUE", "ClientboundDebugBlockValuePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_DEBUG_CHUNK_VALUE", "ClientboundDebugChunkValuePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_DEBUG_ENTITY_VALUE", "ClientboundDebugEntityValuePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_DEBUG_EVENT", "ClientboundDebugEventPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_DEBUG_SAMPLE", "ClientboundDebugSamplePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_DELETE_CHAT", "ClientboundDeleteChatPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_DISCONNECT", "ClientboundDisconnectPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_DISGUISED_CHAT", "ClientboundDisguisedChatPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_ENTITY_EVENT", "ClientboundEntityEventPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_ENTITY_POSITION_SYNC", "ClientboundEntityPositionSyncPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_EXPLODE", "ClientboundExplodePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_FORGET_LEVEL_CHUNK", "ClientboundForgetLevelChunkPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_GAME_EVENT", "ClientboundGameEventPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS", "ClientboundGameTestHighlightPosPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_MOUNT_SCREEN_OPEN", "ClientboundMountScreenOpenPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_HURT_ANIMATION", "ClientboundHurtAnimationPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_INITIALIZE_BORDER", "ClientboundInitializeBorderPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_KEEP_ALIVE", "ClientboundKeepAlivePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT", "ClientboundLevelChunkWithLightPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_LEVEL_EVENT", "ClientboundLevelEventPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_LEVEL_PARTICLES", "ClientboundLevelParticlesPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_LIGHT_UPDATE", "ClientboundLightUpdatePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_LOGIN", "ClientboundLoginPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_MAP_ITEM_DATA", "ClientboundMapItemDataPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_MERCHANT_OFFERS", "ClientboundMerchantOffersPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_MOVE_ENTITY_POS", "ClientboundMoveEntityPacket.Pos.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_MOVE_ENTITY_POS_ROT", "ClientboundMoveEntityPacket.PosRot.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_MOVE_MINECART_ALONG_TRACK", "ClientboundMoveMinecartPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_MOVE_ENTITY_ROT", "ClientboundMoveEntityPacket.Rot.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_MOVE_VEHICLE", "ClientboundMoveVehiclePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_OPEN_BOOK", "ClientboundOpenBookPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_OPEN_SCREEN", "ClientboundOpenScreenPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_OPEN_SIGN_EDITOR", "ClientboundOpenSignEditorPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_PING", "ClientboundPingPacket.STREAM_CODEC"),
            reg("PingPacketTypes.CLIENTBOUND_PONG_RESPONSE", "ClientboundPongResponsePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLACE_GHOST_RECIPE", "ClientboundPlaceGhostRecipePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_ABILITIES", "ClientboundPlayerAbilitiesPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_CHAT", "ClientboundPlayerChatPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_COMBAT_END", "ClientboundPlayerCombatEndPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_COMBAT_ENTER", "ClientboundPlayerCombatEnterPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_COMBAT_KILL", "ClientboundPlayerCombatKillPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_INFO_REMOVE", "ClientboundPlayerInfoRemovePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_INFO_UPDATE", "ClientboundPlayerInfoUpdatePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_LOOK_AT", "ClientboundPlayerLookAtPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_POSITION", "ClientboundPlayerPositionPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PLAYER_ROTATION", "ClientboundPlayerRotationPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_RECIPE_BOOK_ADD", "ClientboundRecipeBookAddPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_RECIPE_BOOK_REMOVE", "ClientboundRecipeBookRemovePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_RECIPE_BOOK_SETTINGS", "ClientboundRecipeBookSettingsPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_REMOVE_ENTITIES", "ClientboundRemoveEntitiesPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_REMOVE_MOB_EFFECT", "ClientboundRemoveMobEffectPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_RESET_SCORE", "ClientboundResetScorePacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_RESOURCE_PACK_POP", "ClientboundResourcePackPopPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_RESOURCE_PACK_PUSH", "ClientboundResourcePackPushPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_RESPAWN", "ClientboundRespawnPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_ROTATE_HEAD", "ClientboundRotateHeadPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SECTION_BLOCKS_UPDATE", "ClientboundSectionBlocksUpdatePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SELECT_ADVANCEMENTS_TAB", "ClientboundSelectAdvancementsTabPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SERVER_DATA", "ClientboundServerDataPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_ACTION_BAR_TEXT", "ClientboundSetActionBarTextPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_BORDER_CENTER", "ClientboundSetBorderCenterPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_BORDER_LERP_SIZE", "ClientboundSetBorderLerpSizePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_BORDER_SIZE", "ClientboundSetBorderSizePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_BORDER_WARNING_DELAY", "ClientboundSetBorderWarningDelayPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_BORDER_WARNING_DISTANCE", "ClientboundSetBorderWarningDistancePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_CAMERA", "ClientboundSetCameraPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_CHUNK_CACHE_CENTER", "ClientboundSetChunkCacheCenterPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_CHUNK_CACHE_RADIUS", "ClientboundSetChunkCacheRadiusPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_CURSOR_ITEM", "ClientboundSetCursorItemPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION", "ClientboundSetDefaultSpawnPositionPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_DISPLAY_OBJECTIVE", "ClientboundSetDisplayObjectivePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_ENTITY_DATA", "ClientboundSetEntityDataPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_ENTITY_LINK", "ClientboundSetEntityLinkPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_ENTITY_MOTION", "ClientboundSetEntityMotionPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_EQUIPMENT", "ClientboundSetEquipmentPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_EXPERIENCE", "ClientboundSetExperiencePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_HEALTH", "ClientboundSetHealthPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_HELD_SLOT", "ClientboundSetHeldSlotPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_OBJECTIVE", "ClientboundSetObjectivePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_PASSENGERS", "ClientboundSetPassengersPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_PLAYER_INVENTORY", "ClientboundSetPlayerInventoryPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_PLAYER_TEAM", "ClientboundSetPlayerTeamPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_SCORE", "ClientboundSetScorePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_SIMULATION_DISTANCE", "ClientboundSetSimulationDistancePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_SUBTITLE_TEXT", "ClientboundSetSubtitleTextPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_TIME", "ClientboundSetTimePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_TITLE_TEXT", "ClientboundSetTitleTextPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SET_TITLES_ANIMATION", "ClientboundSetTitlesAnimationPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SOUND_ENTITY", "ClientboundSoundEntityPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SOUND", "ClientboundSoundPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_START_CONFIGURATION", "ClientboundStartConfigurationPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_STOP_SOUND", "ClientboundStopSoundPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_STORE_COOKIE", "ClientboundStoreCookiePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_SYSTEM_CHAT", "ClientboundSystemChatPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_TAB_LIST", "ClientboundTabListPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_TAG_QUERY", "ClientboundTagQueryPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_TAKE_ITEM_ENTITY", "ClientboundTakeItemEntityPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_TELEPORT_ENTITY", "ClientboundTeleportEntityPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS", "ClientboundTestInstanceBlockStatus.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_TICKING_STATE", "ClientboundTickingStatePacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_TICKING_STEP", "ClientboundTickingStepPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_TRANSFER", "ClientboundTransferPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_UPDATE_ADVANCEMENTS", "ClientboundUpdateAdvancementsPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_UPDATE_ATTRIBUTES", "ClientboundUpdateAttributesPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_UPDATE_MOB_EFFECT", "ClientboundUpdateMobEffectPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_UPDATE_RECIPES", "ClientboundUpdateRecipesPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_UPDATE_TAGS", "ClientboundUpdateTagsPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_PROJECTILE_POWER", "ClientboundProjectilePowerPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_CUSTOM_REPORT_DETAILS", "ClientboundCustomReportDetailsPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_SERVER_LINKS", "ClientboundServerLinksPacket.STREAM_CODEC"),
            reg("GamePacketTypes.CLIENTBOUND_WAYPOINT", "ClientboundTrackedWaypointPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_CLEAR_DIALOG", "ClientboundClearDialogPacket.STREAM_CODEC"),
            reg("CommonPacketTypes.CLIENTBOUND_SHOW_DIALOG", "ClientboundShowDialogPacket.STREAM_CODEC"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct TestContext(bool);

    impl Context for TestContext {
        fn has_infinite_materials(&self) -> bool {
            self.0
        }
    }

    struct MockI32Codec;

    impl StreamCodec<Vec<i32>, i32> for MockI32Codec {
        fn decode(&self, buf: &mut Vec<i32>) -> Result<i32, PacketCodecError> {
            buf.pop()
                .ok_or_else(|| PacketCodecError::SkipDecoder("empty".to_string()))
        }

        fn encode(&self, buf: &mut Vec<i32>, value: &i32) -> Result<(), PacketCodecError> {
            buf.push(*value);
            Ok(())
        }
    }

    struct MockF32Codec;

    impl StreamCodec<Vec<u32>, f32> for MockF32Codec {
        fn decode(&self, buf: &mut Vec<u32>) -> Result<f32, PacketCodecError> {
            buf.pop()
                .map(f32::from_bits)
                .ok_or_else(|| PacketCodecError::SkipDecoder("empty".to_string()))
        }

        fn encode(&self, buf: &mut Vec<u32>, value: &f32) -> Result<(), PacketCodecError> {
            buf.push(value.to_bits());
            Ok(())
        }
    }

    #[test]
    fn decode_rejected_without_infinite_materials() {
        let ctx = TestContext(false);
        let codec = has_infinite_materials_codec::<Vec<i32>, i32, _, _>(MockI32Codec, &ctx);
        let mut buf = vec![42_i32];
        let err = codec.decode(&mut buf).unwrap_err();
        assert_eq!(
            err,
            PacketCodecError::SkipDecoder("Not in creative mode".to_string())
        );
        assert_eq!(buf, vec![42_i32]);
    }

    #[test]
    fn encode_rejected_without_infinite_materials() {
        let ctx = TestContext(false);
        let codec = has_infinite_materials_codec::<Vec<i32>, i32, _, _>(MockI32Codec, &ctx);
        let mut buf = Vec::new();
        let err = codec.encode(&mut buf, &7_i32).unwrap_err();
        assert_eq!(
            err,
            PacketCodecError::SkipEncoder("Not in creative mode".to_string())
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn decode_encode_pass_through_with_infinite_materials_edge_ints() {
        let ctx = TestContext(true);
        let codec = has_infinite_materials_codec::<Vec<i32>, i32, _, _>(MockI32Codec, &ctx);
        let values = [i32::MIN, -1_i32, 0_i32, i32::MAX, i32::MAX.wrapping_add(1)];
        for &v in &values {
            let mut buf = Vec::new();
            codec.encode(&mut buf, &v).unwrap();
            let decoded = codec.decode(&mut buf).unwrap();
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn float_bits_preserved_through_codec() {
        let ctx = TestContext(true);
        let codec = has_infinite_materials_codec::<Vec<u32>, f32, _, _>(MockF32Codec, &ctx);
        let values = [
            0.0_f32,
            -0.0_f32,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            f32::from_bits(0xFFC0_0001),
        ];
        for &v in &values {
            let mut buf = Vec::new();
            codec.encode(&mut buf, &v).unwrap();
            let decoded = codec.decode(&mut buf).unwrap();
            assert_eq!(decoded.to_bits(), v.to_bits());
        }
    }

    #[test]
    fn serverbound_template_contains_creative_slot_modifier() {
        let template = serverbound_template();
        assert_eq!(template.protocol, ConnectionProtocol::Play);
        assert_eq!(template.packets.len(), 66);
        let creative = template
            .packets
            .iter()
            .find(|p| p.packet_type == "GamePacketTypes.SERVERBOUND_SET_CREATIVE_MODE_SLOT")
            .expect("creative slot packet missing");
        assert_eq!(
            creative.stream_codec,
            "ServerboundSetCreativeModeSlotPacket.STREAM_CODEC"
        );
        assert_eq!(creative.modifier, Some("HAS_INFINITE_MATERIALS"));
    }

    #[test]
    fn clientbound_template_has_bundle_and_known_packets() {
        let template = clientbound_template();
        assert_eq!(template.protocol, ConnectionProtocol::Play);
        assert_eq!(template.bundle.packet_type, "GamePacketTypes.CLIENTBOUND_BUNDLE");
        assert_eq!(template.bundle.bundle_factory, "ClientboundBundlePacket::new");
        assert_eq!(template.bundle.delimiter, "ClientboundBundleDelimiterPacket");
        assert!(!template.packets.is_empty());
        let first = &template.packets[0];
        assert_eq!(first.packet_type, "GamePacketTypes.CLIENTBOUND_ADD_ENTITY");
        let last = template.packets.last().unwrap();
        assert_eq!(last.packet_type, "CommonPacketTypes.CLIENTBOUND_SHOW_DIALOG");
    }
}
