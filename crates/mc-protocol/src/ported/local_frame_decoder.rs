//! Port of `net.minecraft.network.LocalFrameDecoder`.

/// Minimal self-contained stub for `net.minecraft.network.HiddenByteBuf`.
pub mod hidden_byte_buf {
    // JAVA: net.minecraft.network.HiddenByteBuf#unpack
    pub fn unpack<T>(msg: T) -> T {
        msg
    }
}

/// Minimal self-contained stub for a Netty-like channel context.
pub trait ChannelHandlerContext<T> {
    fn fire_channel_read(&mut self, msg: T);
}

/// Port of `net.minecraft.network.LocalFrameDecoder`.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalFrameDecoder;

impl LocalFrameDecoder {
    // JAVA: net.minecraft.network.LocalFrameDecoder#channelRead
    pub fn channel_read<T, C: ChannelHandlerContext<T>>(&self, ctx: &mut C, msg: T) {
        ctx.fire_channel_read(hidden_byte_buf::unpack(msg));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestContext<T> {
        received: Vec<T>,
    }

    impl<T> ChannelHandlerContext<T> for TestContext<T> {
        fn fire_channel_read(&mut self, msg: T) {
            self.received.push(msg);
        }
    }

    #[test]
    fn channel_read_forwards_message() {
        let decoder = LocalFrameDecoder;
        let mut ctx = TestContext::default();
        decoder.channel_read(&mut ctx, 42_i32);
        assert_eq!(ctx.received, vec![42_i32]);
    }

    #[test]
    fn channel_read_preserves_negative_and_extreme_integers() {
        let decoder = LocalFrameDecoder;
        let mut ctx = TestContext::default();
        decoder.channel_read(&mut ctx, -1_i32);
        decoder.channel_read(&mut ctx, i32::MIN);
        decoder.channel_read(&mut ctx, i32::MAX);
        assert_eq!(ctx.received, vec![-1_i32, i32::MIN, i32::MAX]);
    }

    #[test]
    fn channel_read_preserves_float_bits() {
        let decoder = LocalFrameDecoder;
        let mut ctx = TestContext::<f64>::default();
        let values = [0.0_f64, -0.0_f64, f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
        for &v in &values {
            decoder.channel_read(&mut ctx, v);
        }
        assert_eq!(ctx.received.len(), values.len());
        for (got, expected) in ctx.received.iter().zip(values.iter()) {
            assert_eq!(got.to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn hidden_byte_buf_unpack_is_identity() {
        let msg = String::from("payload");
        let unpacked = hidden_byte_buf::unpack(msg.clone());
        assert_eq!(unpacked, msg);
    }

    #[test]
    fn channel_read_handles_multiple_messages_in_order() {
        let decoder = LocalFrameDecoder;
        let mut ctx = TestContext::default();
        for i in 0_i64..5_i64 {
            let wrapped = i.wrapping_sub(2); // includes negatives: -2, -1, 0, 1, 2
            decoder.channel_read(&mut ctx, wrapped);
        }
        assert_eq!(ctx.received, vec![-2_i64, -1_i64, 0_i64, 1_i64, 2_i64]);
    }
}
