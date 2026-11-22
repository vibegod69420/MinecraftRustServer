//! Port of `net.minecraft.network.LocalFrameEncoder`.

/// Minimal self-contained stub for `net.minecraft.network.HiddenByteBuf`.
pub mod hidden_byte_buf {
    // JAVA: net.minecraft.network.HiddenByteBuf#pack
    pub fn pack<T>(msg: T) -> T {
        msg
    }
}

/// Minimal self-contained stub for a Netty-like channel context.
pub trait ChannelHandlerContext<T, P> {
    fn write(&mut self, msg: T, promise: P);
}

/// Port of `net.minecraft.network.LocalFrameEncoder`.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalFrameEncoder;

impl LocalFrameEncoder {
    // JAVA: net.minecraft.network.LocalFrameEncoder#write
    pub fn write<T, P, C: ChannelHandlerContext<T, P>>(
        &self,
        ctx: &mut C,
        msg: T,
        promise: P,
    ) {
        ctx.write(hidden_byte_buf::pack(msg), promise);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestContext<T, P> {
        writes: Vec<(T, P)>,
    }

    impl<T, P> ChannelHandlerContext<T, P> for TestContext<T, P> {
        fn write(&mut self, msg: T, promise: P) {
            self.writes.push((msg, promise));
        }
    }

    #[test]
    fn write_forwards_message_and_promise() {
        let encoder = LocalFrameEncoder;
        let mut ctx = TestContext::default();
        encoder.write(&mut ctx, 123_i32, true);
        assert_eq!(ctx.writes, vec![(123_i32, true)]);
    }

    #[test]
    fn write_preserves_negative_and_extreme_integers() {
        let encoder = LocalFrameEncoder;
        let mut ctx = TestContext::default();
        encoder.write(&mut ctx, -1_i32, 1_u8);
        encoder.write(&mut ctx, i32::MIN, 2_u8);
        encoder.write(&mut ctx, i32::MAX, 3_u8);
        assert_eq!(
            ctx.writes,
            vec![(-1_i32, 1_u8), (i32::MIN, 2_u8), (i32::MAX, 3_u8)]
        );
    }

    #[test]
    fn write_preserves_float_bits() {
        let encoder = LocalFrameEncoder;
        let mut ctx = TestContext::<f32, i32>::default();
        let values = [0.0_f32, -0.0_f32, f32::NAN, f32::INFINITY, f32::NEG_INFINITY];
        for (i, &v) in values.iter().enumerate() {
            encoder.write(&mut ctx, v, i as i32);
        }
        assert_eq!(ctx.writes.len(), values.len());
        for ((got, _), expected) in ctx.writes.iter().zip(values.iter()) {
            assert_eq!(got.to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn hidden_byte_buf_pack_is_identity() {
        let msg = vec![1_u8, 2, 3];
        let packed = hidden_byte_buf::pack(msg.clone());
        assert_eq!(packed, msg);
    }

    #[test]
    fn write_handles_multiple_messages_in_order() {
        let encoder = LocalFrameEncoder;
        let mut ctx = TestContext::default();
        for i in 0_i64..4_i64 {
            let wrapped = i.wrapping_mul(2).wrapping_sub(3); // -3, -1, 1, 3
            encoder.write(&mut ctx, wrapped, i);
        }
        assert_eq!(
            ctx.writes,
            vec![(-3_i64, 0_i64), (-1_i64, 1_i64), (1_i64, 2_i64), (3_i64, 3_i64)]
        );
    }
}
