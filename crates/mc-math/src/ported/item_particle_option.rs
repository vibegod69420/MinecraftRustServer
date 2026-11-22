#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParticleType {
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    pub item_id: u32,
    pub count: i32,
}

impl ItemStack {
    pub fn new(item_id: u32, count: i32) -> Self {
        Self { item_id, count }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item_id == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemParticleOption {
    particle_type: ParticleType,
    item_stack: ItemStack,
}

impl ItemParticleOption {
    // JAVA: net.minecraft.core.particles.ItemParticleOption#<init>
    pub fn new(particle_type: ParticleType, item_stack: ItemStack) -> Self {
        if item_stack.is_empty() {
            panic!("Empty stacks are not allowed");
        }
        Self {
            particle_type,
            item_stack,
        }
    }

    // JAVA: net.minecraft.core.particles.ItemParticleOption#getType
    pub fn get_type(&self) -> ParticleType {
        self.particle_type
    }

    // JAVA: net.minecraft.core.particles.ItemParticleOption#getItem
    pub fn get_item(&self) -> &ItemStack {
        &self.item_stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(id: u32) -> ParticleType {
        ParticleType { id }
    }

    #[test]
    fn new_and_getters() {
        let stack = ItemStack::new(1, 64);
        let opt = ItemParticleOption::new(pt(5), stack.clone());
        assert_eq!(opt.get_type(), pt(5));
        assert_eq!(opt.get_item(), &stack);
    }

    #[test]
    #[should_panic(expected = "Empty stacks are not allowed")]
    fn empty_count_panics() {
        let _ = ItemParticleOption::new(pt(1), ItemStack::new(1, 0));
    }

    #[test]
    #[should_panic(expected = "Empty stacks are not allowed")]
    fn air_item_panics() {
        let _ = ItemParticleOption::new(pt(1), ItemStack::new(0, 1));
    }

    #[test]
    #[should_panic(expected = "Empty stacks are not allowed")]
    fn negative_count_panics() {
        let _ = ItemParticleOption::new(pt(1), ItemStack::new(2, -1));
    }

    #[test]
    fn boundary_non_empty_stack() {
        let stack = ItemStack::new(42, 1);
        let opt = ItemParticleOption::new(pt(9), stack.clone());
        assert_eq!(opt.get_item().item_id, 42);
        assert_eq!(opt.get_item().count, 1);
        assert!(!opt.get_item().is_empty());
    }
}
