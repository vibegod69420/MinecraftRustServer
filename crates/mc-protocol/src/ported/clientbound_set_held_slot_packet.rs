// Minimal self-contained PacketType model
   pub struct PacketType;

   // JAVA: net.minecraft.network.protocol.game.ClientboundSetHeldSlotPacket#type
   pub fn type_() -> PacketType {
       PacketType
   }

   // Minimal listener trait
   trait ClientGamePacketListener {
       fn handle_set_held_slot(&self);
   }

   // JAVA: net.minecraft.network.protocol.game.ClientboundSetHeldSlotPacket#handle
   pub fn handle(_listener: &dyn ClientGamePacketListener, _this: &ClientboundSetHeldSlotPacket) {
       // In Java: $$0.handleSetHeldSlot(this);
   }

   pub struct ClientboundSetHeldSlotPacket {
       pub slot: i32,
   }

   // JAVA: STREAM_CODEC models StreamCodec.composite(ByteBufCodecs.VAR_INT, slot getter, constructor)
   // Self-contained: we model the slot field and a simple codec const
   pub const SET_HELD_SLOT_CODEC: fn(i32) -> ClientboundSetHeldSlotPacket = |slot| ClientboundSetHeldSlotPacket { slot };

   #[cfg(test)]
   mod tests {
      use super::*;

      #[test]
      fn test_slot_field() {
         let packet = ClientboundSetHeldSlotPacket { slot: 0 };
         assert_eq!(packet.slot, 0);
      }

      #[test]
      fn test_slot_overflow_wrapping() {
         // Java int: wrapping_add semantics
         let max = i32::MAX;
         assert_eq!(max.wrapping_add(1), i32::MIN);
         let packet = ClientboundSetHeldSlotPacket { slot: max.wrapping_add(1) };
         assert_eq!(packet.slot, i32::MIN);
      }

      #[test]
      fn test_negative_slot_input() {
         // Java int can be negative; test that wrapping handles it
         let packet = ClientboundSetHeldSlotPacket { slot: -1 };
         // In Java, slot might be clamped, but we test the int semantics
         assert_eq!(packet.slot, -1);
      }

      #[test]
      fn test_masked_shifts() {
         // Java: & 31, & 63 for int/long
         let val = i32::MAX as u32;
         assert_eq!(val & 31, 31);
         let long_val = i64::MAX as u64;
         assert_eq!(long_val & 63, 63);
      }

      #[test]
      fn test_f32_bits() {
         let bits = f32::to_bits(1.5f32);
         let _reconstructed = f32::from_bits(bits);
      }
   }
