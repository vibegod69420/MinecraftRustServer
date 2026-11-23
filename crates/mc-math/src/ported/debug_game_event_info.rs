//! Faithful port of `net.minecraft.util.debug.DebugGameEventInfo`.

/// Minimal self-contained stand-in for `net.minecraft.core.Holder<T>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Holder<T> {
    pub value: T,
}

/// Minimal self-contained stand-in for `net.minecraft.world.level.gameevent.GameEvent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEvent {
    pub id: String,
}

/// Minimal self-contained stand-in for `net.minecraft.world.phys.Vec3`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Faithful data representation of `DebugGameEventInfo(Holder<GameEvent> event, Vec3 pos)`.
///
/// Java STREAM_CODEC composition:
/// `StreamCodec.composite(
///   ByteBufCodecs.holderRegistry(Registries.GAME_EVENT), DebugGameEventInfo::event,
///   Vec3.STREAM_CODEC, DebugGameEventInfo::pos,
///   DebugGameEventInfo::new
/// )`
#[derive(Debug, Clone, PartialEq)]
pub struct DebugGameEventInfo {
    pub event: Holder<GameEvent>,
    pub pos: Vec3,
}

impl DebugGameEventInfo {
    // JAVA: net.minecraft.util.debug.DebugGameEventInfo#<init>
    pub fn new(event: Holder<GameEvent>, pos: Vec3) -> Self {
        Self { event, pos }
    }

    // JAVA: net.minecraft.util.debug.DebugGameEventInfo#event
    pub fn event(&self) -> &Holder<GameEvent> {
        &self.event
    }

    // JAVA: net.minecraft.util.debug.DebugGameEventInfo#pos
    pub fn pos(&self) -> Vec3 {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_accessors() {
        let event = Holder {
            value: GameEvent {
                id: "minecraft:step".to_string(),
            },
        };
        let pos = Vec3 {
            x: 1.5,
            y: -2.0,
            z: 3.25,
        };
        let info = DebugGameEventInfo::new(event.clone(), pos);
        assert_eq!(info.event(), &event);
        assert_eq!(info.pos(), pos);
    }

    #[test]
    fn supports_negative_and_large_coordinates() {
        let info = DebugGameEventInfo::new(
            Holder {
                value: GameEvent {
                    id: "test:event".to_string(),
                },
            },
            Vec3 {
                x: -1.0e12,
                y: f64::MIN_POSITIVE,
                z: 0.0,
            },
        );
        assert_eq!(info.pos().x, -1.0e12);
        assert_eq!(info.pos().y, f64::MIN_POSITIVE);
        assert_eq!(info.pos().z, 0.0);
    }

    #[test]
    fn clone_and_equality() {
        let a = DebugGameEventInfo::new(
            Holder {
                value: GameEvent {
                    id: "a".to_string(),
                },
            },
            Vec3 {
                x: 0.0,
                y: 1.0,
                z: 2.0,
            },
        );
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn float_bits_preserved_for_special_values() {
        let pos = Vec3 {
            x: f64::NAN,
            y: f64::INFINITY,
            z: f64::NEG_INFINITY,
        };
        let info = DebugGameEventInfo::new(
            Holder {
                value: GameEvent {
                    id: "special".to_string(),
                },
            },
            pos,
        );
        assert!(info.pos().x.is_nan());
        assert!(info.pos().y.is_infinite() && info.pos().y.is_sign_positive());
        assert!(info.pos().z.is_infinite() && info.pos().z.is_sign_negative());
        assert_eq!(info.pos().x.to_bits(), f64::NAN.to_bits());
    }

    #[test]
    fn exact_float_bits_round_trip() {
        let x_bits: u64 = 0xBFF8_0000_0000_0001; // negative, near -1.5 with payload bits
        let y_bits: u64 = 0x8000_0000_0000_0000; // -0.0
        let z_bits: u64 = 0x7FF0_0000_0000_0001; // signaling/payload NaN bit pattern
        let pos = Vec3 {
            x: f64::from_bits(x_bits),
            y: f64::from_bits(y_bits),
            z: f64::from_bits(z_bits),
        };
        let info = DebugGameEventInfo::new(
            Holder {
                value: GameEvent {
                    id: "bits".to_string(),
                },
            },
            pos,
        );
        assert_eq!(info.pos().x.to_bits(), x_bits);
        assert_eq!(info.pos().y.to_bits(), y_bits);
        assert_eq!(info.pos().z.to_bits(), z_bits);
    }
}
