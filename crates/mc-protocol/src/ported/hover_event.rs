use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;

/// Minimal self-contained UUID representation for the hover event port.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Uuid {
    pub most: u64,
    pub least: u64,
}

impl Uuid {
    /// Returns the Java `UUID.hashCode()` value.
    pub fn java_hash_code(&self) -> i32 {
        let most = self.most as i64;
        let least = self.least as i64;
        let hilo = most ^ least;
        ((hilo >> 32) as i32) ^ (hilo as i32)
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Java UUID.toString() format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (self.most >> 32) as u32,
            ((self.most >> 16) & 0xffff) as u16,
            (self.most & 0xffff) as u16,
            (self.least >> 48) as u16,
            (self.least & 0xffff_ffff_ffff) as u64
        )
    }
}

fn java_hash_string(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(c as i32);
    }
    h
}

/// Minimal self-contained chat component representation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Component {
    Literal(String),
    Translatable { key: String, args: Vec<Component> },
}

impl Component {
    // JAVA: net.minecraft.network.chat.Component#literal
    pub fn literal(s: impl Into<String>) -> Self {
        Component::Literal(s.into())
    }

    // JAVA: net.minecraft.network.chat.Component#translatable
    pub fn translatable(key: impl Into<String>, args: Vec<Component>) -> Self {
        Component::Translatable {
            key: key.into(),
            args,
        }
    }

    fn java_hash_code(&self) -> i32 {
        match self {
            Component::Literal(s) => java_hash_string(s),
            Component::Translatable { key, args } => {
                let mut h = java_hash_string(key);
                for arg in args {
                    h = h.wrapping_mul(31).wrapping_add(arg.java_hash_code());
                }
                h
            }
        }
    }
}

/// Minimal self-contained entity type representation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityType {
    pub description: Component,
}

impl EntityType {
    // JAVA: net.minecraft.world.entity.EntityType#getDescription
    pub fn get_description(&self) -> &Component {
        &self.description
    }

    fn java_hash_code(&self) -> i32 {
        self.description.java_hash_code()
    }
}

/// Minimal self-contained item stack representation.
#[derive(Clone, Debug)]
pub struct ItemStack {
    pub id: String,
    pub count: i32,
}

impl ItemStack {
    // JAVA: net.minecraft.world.item.ItemStack#copy
    pub fn copy(&self) -> Self {
        self.clone()
    }

    // JAVA: net.minecraft.world.item.ItemStack#matches
    pub fn matches(a: &ItemStack, b: &ItemStack) -> bool {
        a.id == b.id && a.count == b.count
    }

    // JAVA: net.minecraft.world.item.ItemStack#hashItemAndComponents
    pub fn hash_item_and_components(stack: &ItemStack) -> i32 {
        let mut h = java_hash_string(&stack.id);
        h = h.wrapping_mul(31).wrapping_add(stack.count);
        h
    }
}

/// Port of `net.minecraft.network.chat.HoverEvent.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    ShowText,
    ShowItem,
    ShowEntity,
}

impl Action {
    // JAVA: net.minecraft.network.chat.HoverEvent$Action#isAllowedFromServer
    pub fn is_allowed_from_server(&self) -> bool {
        match self {
            Action::ShowText | Action::ShowItem | Action::ShowEntity => true,
        }
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$Action#getSerializedName
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Action::ShowText => "show_text",
            Action::ShowItem => "show_item",
            Action::ShowEntity => "show_entity",
        }
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$Action#filterForSerialization
    pub fn filter_for_serialization(&self) -> Result<Self, String> {
        if !self.is_allowed_from_server() {
            Err(format!("Action not allowed: {}", self))
        } else {
            Ok(*self)
        }
    }
}

impl fmt::Display for Action {
    // JAVA: net.minecraft.network.chat.HoverEvent$Action#toString
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<action {}>", self.get_serialized_name())
    }
}

/// Port of `net.minecraft.network.chat.HoverEvent.EntityTooltipInfo`.
#[derive(Debug)]
pub struct EntityTooltipInfo {
    pub entity_type: EntityType,
    pub uuid: Uuid,
    pub name: Option<Component>,
    lines_cache: OnceLock<Vec<Component>>,
}

impl EntityTooltipInfo {
    // JAVA: net.minecraft.network.chat.HoverEvent$EntityTooltipInfo#<init>
    pub fn new(entity_type: EntityType, uuid: Uuid, name: Option<Component>) -> Self {
        Self {
            entity_type,
            uuid,
            name,
            lines_cache: OnceLock::new(),
        }
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$EntityTooltipInfo#getTooltipLines
    pub fn get_tooltip_lines(&self) -> &Vec<Component> {
        self.lines_cache.get_or_init(|| {
            let mut lines = Vec::new();
            if let Some(name) = &self.name {
                lines.push(name.clone());
            }
            lines.push(Component::translatable(
                "gui.entity_tooltip.type",
                vec![self.entity_type.get_description().clone()],
            ));
            lines.push(Component::literal(self.uuid.to_string()));
            lines
        })
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$EntityTooltipInfo#hashCode
    pub fn java_hash_code(&self) -> i32 {
        let mut h = self.entity_type.java_hash_code();
        h = h.wrapping_mul(31).wrapping_add(self.uuid.java_hash_code());
        let name_hash = match &self.name {
            Some(c) => c.java_hash_code(),
            None => 0, // Java Optional.empty().hashCode() == 0
        };
        h.wrapping_mul(31).wrapping_add(name_hash)
    }
}

impl Clone for EntityTooltipInfo {
    fn clone(&self) -> Self {
        Self {
            entity_type: self.entity_type.clone(),
            uuid: self.uuid,
            name: self.name.clone(),
            lines_cache: OnceLock::new(),
        }
    }
}

impl PartialEq for EntityTooltipInfo {
    // JAVA: net.minecraft.network.chat.HoverEvent$EntityTooltipInfo#equals
    fn eq(&self, other: &Self) -> bool {
        self.entity_type == other.entity_type
            && self.uuid == other.uuid
            && self.name == other.name
    }
}

impl Eq for EntityTooltipInfo {}

impl Hash for EntityTooltipInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entity_type.hash(state);
        self.uuid.hash(state);
        self.name.hash(state);
    }
}

/// Port of `net.minecraft.network.chat.HoverEvent.ShowEntity`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShowEntity {
    pub entity: EntityTooltipInfo,
}

impl ShowEntity {
    // JAVA: net.minecraft.network.chat.HoverEvent$ShowEntity#<init>
    pub fn new(entity: EntityTooltipInfo) -> Self {
        Self { entity }
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$ShowEntity#action
    pub fn action(&self) -> Action {
        Action::ShowEntity
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$ShowEntity#entity
    pub fn entity(&self) -> &EntityTooltipInfo {
        &self.entity
    }
}

/// Port of `net.minecraft.network.chat.HoverEvent.ShowItem`.
#[derive(Clone, Debug)]
pub struct ShowItem {
    pub item: ItemStack,
}

impl ShowItem {
    // JAVA: net.minecraft.network.chat.HoverEvent$ShowItem#<init>
    pub fn new(item: ItemStack) -> Self {
        Self { item: item.copy() }
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$ShowItem#action
    pub fn action(&self) -> Action {
        Action::ShowItem
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$ShowItem#item
    pub fn item(&self) -> &ItemStack {
        &self.item
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$ShowItem#hashCode
    pub fn java_hash_code(&self) -> i32 {
        ItemStack::hash_item_and_components(&self.item)
    }
}

impl PartialEq for ShowItem {
    // JAVA: net.minecraft.network.chat.HoverEvent$ShowItem#equals
    fn eq(&self, other: &Self) -> bool {
        ItemStack::matches(&self.item, &other.item)
    }
}

impl Eq for ShowItem {}

impl Hash for ShowItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.java_hash_code().hash(state);
    }
}

/// Port of `net.minecraft.network.chat.HoverEvent.ShowText`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShowText {
    pub value: Component,
}

impl ShowText {
    // JAVA: net.minecraft.network.chat.HoverEvent$ShowText#<init>
    pub fn new(value: Component) -> Self {
        Self { value }
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$ShowText#action
    pub fn action(&self) -> Action {
        Action::ShowText
    }

    // JAVA: net.minecraft.network.chat.HoverEvent$ShowText#value
    pub fn value(&self) -> &Component {
        &self.value
    }
}

/// Port of `net.minecraft.network.chat.HoverEvent`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HoverEvent {
    ShowText(ShowText),
    ShowItem(ShowItem),
    ShowEntity(ShowEntity),
}

impl HoverEvent {
    // JAVA: net.minecraft.network.chat.HoverEvent#action
    pub fn action(&self) -> Action {
        match self {
            HoverEvent::ShowText(_) => Action::ShowText,
            HoverEvent::ShowItem(_) => Action::ShowItem,
            HoverEvent::ShowEntity(_) => Action::ShowEntity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_names_display_and_serialization_filter() {
        assert_eq!(Action::ShowText.get_serialized_name(), "show_text");
        assert_eq!(Action::ShowItem.get_serialized_name(), "show_item");
        assert_eq!(Action::ShowEntity.get_serialized_name(), "show_entity");
        assert_eq!(Action::ShowText.to_string(), "<action show_text>");
        assert!(Action::ShowItem.is_allowed_from_server());
        assert_eq!(
            Action::ShowEntity.filter_for_serialization(),
            Ok(Action::ShowEntity)
        );
    }

    #[test]
    fn entity_tooltip_lines_cached_and_content() {
        let info = EntityTooltipInfo::new(
            EntityType {
                description: Component::literal("Pig"),
            },
            Uuid { most: 0, least: 1 },
            Some(Component::literal("Bob")),
        );
        let lines1 = info.get_tooltip_lines();
        let lines2 = info.get_tooltip_lines();
        assert_eq!(lines1.len(), 3);
        assert_eq!(lines1[0], Component::literal("Bob"));
        assert_eq!(
            lines1[1],
            Component::translatable("gui.entity_tooltip.type", vec![Component::literal("Pig")])
        );
        assert_eq!(lines1[2], Component::literal("00000000-0000-0000-0000-000000000001"));
        assert!(std::ptr::eq(lines1, lines2));
    }

    #[test]
    fn entity_tooltip_no_name_and_uuid_format() {
        let uuid = Uuid {
            most: 0x0123_4567_89ab_cdef,
            least: 0xfedc_ba98_7654_3210,
        };
        let info = EntityTooltipInfo::new(
            EntityType {
                description: Component::literal("Zombie"),
            },
            uuid,
            None,
        );
        let lines = info.get_tooltip_lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[1],
            Component::literal("01234567-89ab-cdef-fedc-ba9876543210")
        );
    }

    #[test]
    fn entity_tooltip_equals_and_hash_overflow() {
        let uuid = Uuid {
            most: u64::MAX,
            least: u64::MAX,
        };
        let a = EntityTooltipInfo::new(
            EntityType {
                description: Component::literal("A"),
            },
            uuid,
            Some(Component::literal("name")),
        );
        let b = EntityTooltipInfo::new(
            EntityType {
                description: Component::literal("A"),
            },
            uuid,
            Some(Component::literal("name")),
        );
        let c = EntityTooltipInfo::new(
            EntityType {
                description: Component::literal("B"),
            },
            uuid,
            Some(Component::literal("name")),
        );
        assert_eq!(a, b);
        assert_ne!(a, c);
        // Java hashCode uses wrapping 31-multiplication; must not panic on overflow/negative.
        let h1 = a.java_hash_code();
        let h2 = b.java_hash_code();
        assert_eq!(h1, h2);
        let _ = h1.wrapping_mul(31).wrapping_add(i32::MIN);
    }

    #[test]
    fn show_item_copies_on_construct_and_matches_by_value() {
        let original = ItemStack {
            id: "minecraft:stone".to_string(),
            count: -1, // negative edge case
        };
        let show = ShowItem::new(original.clone());
        assert_eq!(show.item().id, "minecraft:stone");
        assert_eq!(show.item().count, -1);

        let same = ShowItem::new(ItemStack {
            id: "minecraft:stone".to_string(),
            count: -1,
        });
        let different = ShowItem::new(ItemStack {
            id: "minecraft:dirt".to_string(),
            count: -1,
        });
        assert_eq!(show, same);
        assert_ne!(show, different);
        assert_eq!(show.java_hash_code(), same.java_hash_code());
    }

    #[test]
    fn hover_event_action_dispatch() {
        let text = HoverEvent::ShowText(ShowText::new(Component::literal("hi")));
        assert_eq!(text.action(), Action::ShowText);

        let item = HoverEvent::ShowItem(ShowItem::new(ItemStack {
            id: "minecraft:stone".to_string(),
            count: 1,
        }));
        assert_eq!(item.action(), Action::ShowItem);

        let entity = HoverEvent::ShowEntity(ShowEntity::new(EntityTooltipInfo::new(
            EntityType {
                description: Component::literal("E"),
            },
            Uuid { most: 0, least: 0 },
            None,
        )));
        assert_eq!(entity.action(), Action::ShowEntity);
    }
}
