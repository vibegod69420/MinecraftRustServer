// JAVA: net.minecraft.network.HandlerNames
pub struct HandlerNames;

impl HandlerNames {
    pub const DECOMPRESS: &'static str = "decompress";
    pub const COMPRESS: &'static str = "compress";
    pub const DECODER: &'static str = "decoder";
    pub const ENCODER: &'static str = "encoder";
    pub const INBOUND_CONFIG: &'static str = "inbound_config";
    pub const OUTBOUND_CONFIG: &'static str = "outbound_config";
    pub const SPLITTER: &'static str = "splitter";
    pub const PREPENDER: &'static str = "prepender";
    pub const DECRYPT: &'static str = "decrypt";
    pub const ENCRYPT: &'static str = "encrypt";
    pub const UNBUNDLER: &'static str = "unbundler";
    pub const BUNDLER: &'static str = "bundler";
    pub const PACKET_HANDLER: &'static str = "packet_handler";
    pub const TIMEOUT: &'static str = "timeout";
    pub const LEGACY_QUERY: &'static str = "legacy_query";
    pub const LATENCY: &'static str = "latency";
}

#[cfg(test)]
mod tests {
    use super::HandlerNames;

    #[test]
    fn test_core_handler_names() {
        assert_eq!(HandlerNames::DECOMPRESS, "decompress");
        assert_eq!(HandlerNames::COMPRESS, "compress");
        assert_eq!(HandlerNames::DECODER, "decoder");
        assert_eq!(HandlerNames::ENCODER, "encoder");
    }

    #[test]
    fn test_pipeline_handler_names() {
        assert_eq!(HandlerNames::SPLITTER, "splitter");
        assert_eq!(HandlerNames::PREPENDER, "prepender");
        assert_eq!(HandlerNames::PACKET_HANDLER, "packet_handler");
        assert_eq!(HandlerNames::TIMEOUT, "timeout");
    }

    #[test]
    fn test_security_and_misc_names() {
        assert_eq!(HandlerNames::DECRYPT, "decrypt");
        assert_eq!(HandlerNames::ENCRYPT, "encrypt");
        assert_eq!(HandlerNames::LEGACY_QUERY, "legacy_query");
        assert_eq!(HandlerNames::LATENCY, "latency");
    }

    #[test]
    fn test_bundler_names() {
        assert_eq!(HandlerNames::UNBUNDLER, "unbundler");
        assert_eq!(HandlerNames::BUNDLER, "bundler");
        assert_eq!(HandlerNames::INBOUND_CONFIG, "inbound_config");
        assert_eq!(HandlerNames::OUTBOUND_CONFIG, "outbound_config");
    }
}
