/// Faithful port of `net.minecraft.util.debug.DebugEntityBlockIntersection`.
///
/// `STREAM_CODEC` is a Java networking codec binding; the codec framework is not
/// ported in `mc-math`. The `BY_ID` lookup is exposed as `by_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugEntityBlockIntersection {
    InBlock,
    InFluid,
    InAir,
}

impl DebugEntityBlockIntersection {
    // JAVA: net.minecraft.util.debug.DebugEntityBlockIntersection#color
    pub fn color(self) -> i32 {
        match self {
            Self::InBlock => 1_610_678_016,
            Self::InFluid => 1_610_612_991,
            Self::InAir => 1_613_968_179,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugEntityBlockIntersection#id
    pub fn id(self) -> i32 {
        match self {
            Self::InBlock => 0,
            Self::InFluid => 1,
            Self::InAir => 2,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugEntityBlockIntersection#BY_ID
    pub fn by_id(id: i32) -> Self {
        match id {
            1 => Self::InFluid,
            2 => Self::InAir,
            _ => Self::InBlock, // ByIdMap.OutOfBoundsStrategy.ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DebugEntityBlockIntersection;

    #[test]
    fn color_and_id_values_match_java() {
        assert_eq!(DebugEntityBlockIntersection::InBlock.id(), 0);
        assert_eq!(DebugEntityBlockIntersection::InFluid.id(), 1);
        assert_eq!(DebugEntityBlockIntersection::InAir.id(), 2);

        assert_eq!(DebugEntityBlockIntersection::InBlock.color(), 1_610_678_016);
        assert_eq!(DebugEntityBlockIntersection::InFluid.color(), 1_610_612_991);
        assert_eq!(DebugEntityBlockIntersection::InAir.color(), 1_613_968_179);
    }

    #[test]
    fn by_id_valid_values() {
        assert_eq!(
            DebugEntityBlockIntersection::by_id(0),
            DebugEntityBlockIntersection::InBlock
        );
        assert_eq!(
            DebugEntityBlockIntersection::by_id(1),
            DebugEntityBlockIntersection::InFluid
        );
        assert_eq!(
            DebugEntityBlockIntersection::by_id(2),
            DebugEntityBlockIntersection::InAir
        );
    }

    #[test]
    fn by_id_out_of_bounds_uses_zero_strategy() {
        assert_eq!(
            DebugEntityBlockIntersection::by_id(-1),
            DebugEntityBlockIntersection::InBlock
        );
        assert_eq!(
            DebugEntityBlockIntersection::by_id(3),
            DebugEntityBlockIntersection::InBlock
        );
        assert_eq!(
            DebugEntityBlockIntersection::by_id(i32::MIN),
            DebugEntityBlockIntersection::InBlock
        );
        assert_eq!(
            DebugEntityBlockIntersection::by_id(i32::MAX),
            DebugEntityBlockIntersection::InBlock
        );
    }

    #[test]
    fn enum_is_copy_and_eq() {
        let a = DebugEntityBlockIntersection::InFluid;
        let b = a;
        assert_eq!(a, b);
        assert_ne!(a, DebugEntityBlockIntersection::InAir);
    }
}
