//! Faithful port of `net.minecraft.util.debug.DebugGoalInfo`.

/// Faithful data representation of `DebugGoalInfo.DebugGoal(int priority, boolean isRunning, String name)`.
///
/// Java STREAM_CODEC composition:
/// `StreamCodec.composite(
///   ByteBufCodecs.VAR_INT, DebugGoal::priority,
///   ByteBufCodecs.BOOL, DebugGoal::isRunning,
///   ByteBufCodecs.stringUtf8(255), DebugGoal::name,
///   DebugGoal::new
/// )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugGoal {
    pub priority: i32,
    pub is_running: bool,
    pub name: String,
}

impl DebugGoal {
    // JAVA: net.minecraft.util.debug.DebugGoalInfo$DebugGoal#<init>
    pub fn new(priority: i32, is_running: bool, name: String) -> Self {
        Self {
            priority,
            is_running,
            name,
        }
    }

    // JAVA: net.minecraft.util.debug.DebugGoalInfo$DebugGoal#priority
    pub fn priority(&self) -> i32 {
        self.priority
    }

    // JAVA: net.minecraft.util.debug.DebugGoalInfo$DebugGoal#isRunning
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    // JAVA: net.minecraft.util.debug.DebugGoalInfo$DebugGoal#name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Faithful data representation of `DebugGoalInfo(List<DebugGoalInfo.DebugGoal> goals)`.
///
/// Java STREAM_CODEC composition:
/// `StreamCodec.composite(
///   DebugGoalInfo.DebugGoal.STREAM_CODEC.apply(ByteBufCodecs.list()), DebugGoalInfo::goals,
///   DebugGoalInfo::new
/// )`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugGoalInfo {
    pub goals: Vec<DebugGoal>,
}

impl DebugGoalInfo {
    // JAVA: net.minecraft.util.debug.DebugGoalInfo#<init>
    pub fn new(goals: Vec<DebugGoal>) -> Self {
        Self { goals }
    }

    // JAVA: net.minecraft.util.debug.DebugGoalInfo#goals
    pub fn goals(&self) -> &[DebugGoal] {
        &self.goals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_goal_new_and_accessors() {
        let goal = DebugGoal::new(3, true, "MoveToBlock".to_string());
        assert_eq!(goal.priority(), 3);
        assert!(goal.is_running());
        assert_eq!(goal.name(), "MoveToBlock");
    }

    #[test]
    fn debug_goal_info_round_trip() {
        let goals = vec![
            DebugGoal::new(0, false, "A".to_string()),
            DebugGoal::new(-1, true, "B".to_string()),
        ];
        let info = DebugGoalInfo::new(goals.clone());
        assert_eq!(info.goals(), goals.as_slice());
    }

    #[test]
    fn debug_goal_priority_edge_cases() {
        let min_goal = DebugGoal::new(i32::MIN, false, "min".to_string());
        let max_goal = DebugGoal::new(i32::MAX, true, "max".to_string());
        assert_eq!(min_goal.priority(), i32::MIN);
        assert_eq!(max_goal.priority(), i32::MAX);

        // Java int semantics are represented by i32 storage; wrapping arithmetic example.
        let wrapped = min_goal.priority().wrapping_sub(1);
        assert_eq!(wrapped, i32::MAX);
    }

    #[test]
    fn empty_goals_and_clone_equality() {
        let info = DebugGoalInfo::new(vec![]);
        assert!(info.goals().is_empty());
        let cloned = info.clone();
        assert_eq!(info, cloned);
    }

    #[test]
    fn debug_goal_name_and_running_flags() {
        let g1 = DebugGoal::new(1, false, String::new());
        let g2 = DebugGoal::new(2, true, "a".repeat(255));
        assert!(!g1.is_running());
        assert_eq!(g1.name(), "");
        assert!(g2.is_running());
        assert_eq!(g2.name().len(), 255);
        assert_eq!(g2.name().as_bytes()[0], b'a');
    }
}
