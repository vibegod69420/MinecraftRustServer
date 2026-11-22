/// Minimal stand-in for `net.minecraft.network.FriendlyByteBuf`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FriendlyByteBuf {
    pub data: Vec<u8>,
}

impl FriendlyByteBuf {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

pub trait CustomQueryAnswerPayload {
    // JAVA: net.minecraft.network.protocol.login.custom.CustomQueryAnswerPayload#write
    fn write(&self, buf: &mut FriendlyByteBuf);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DiscardedQueryAnswerPayload;

impl DiscardedQueryAnswerPayload {
    pub const INSTANCE: DiscardedQueryAnswerPayload = DiscardedQueryAnswerPayload;
}

impl CustomQueryAnswerPayload for DiscardedQueryAnswerPayload {
    // JAVA: net.minecraft.network.protocol.login.custom.DiscardedQueryAnswerPayload#write
    fn write(&self, _buf: &mut FriendlyByteBuf) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_is_default() {
        assert_eq!(
            DiscardedQueryAnswerPayload::INSTANCE,
            DiscardedQueryAnswerPayload::default()
        );
    }

    #[test]
    fn write_is_noop() {
        let payload = DiscardedQueryAnswerPayload::INSTANCE;
        let mut buf = FriendlyByteBuf::new();
        payload.write(&mut buf);
        assert!(buf.data.is_empty());
    }

    #[test]
    fn write_preserves_existing_buffer_contents() {
        let payload = DiscardedQueryAnswerPayload;
        let mut buf = FriendlyByteBuf {
            data: vec![0xFF, 0x00, 0x7F],
        };
        payload.write(&mut buf);
        assert_eq!(buf.data, vec![0xFF, 0x00, 0x7F]);
    }

    #[test]
    fn copy_and_equality() {
        let a = DiscardedQueryAnswerPayload::INSTANCE;
        let b = a;
        let c = DiscardedQueryAnswerPayload;
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn friendly_byte_buf_default_is_empty() {
        let buf = FriendlyByteBuf::default();
        assert!(buf.data.is_empty());
    }
}
