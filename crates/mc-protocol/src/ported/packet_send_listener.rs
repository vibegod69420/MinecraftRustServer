use std::sync::Arc;

// JAVA: net.minecraft.network.PacketSendListener
pub struct PacketSendListener;

pub trait PacketLike {
    fn packet_type_name(&self) -> &'static str;
}

pub type PacketSendFutureResult = Result<(), String>;

pub type PacketSendCallback = Arc<dyn Fn(PacketSendFutureResult) + Send + Sync>;

impl PacketSendListener {
    // JAVA: net.minecraft.network.PacketSendListener#thenRun
    pub fn then_run<F>(on_run: F) -> PacketSendCallback
    where
        F: Fn() + Send + Sync + 'static,
    {
        Arc::new(move |result: PacketSendFutureResult| {
            on_run();
            if let Err(err) = result {
                eprintln!("packet send failed after then_run: {err}");
            }
        })
    }

    // JAVA: net.minecraft.network.PacketSendListener#exceptionallySend
    pub fn exceptionally_send<F, P>(fallback_packet: F) -> PacketSendCallback
    where
        F: Fn() -> Option<P> + Send + Sync + 'static,
        P: PacketLike + 'static,
    {
        Arc::new(move |result: PacketSendFutureResult| {
            if let Err(err) = result {
                if let Some(pkt) = fallback_packet() {
                    eprintln!(
                        "Failed to deliver packet, sending fallback {} ({err})",
                        pkt.packet_type_name()
                    );
                } else {
                    eprintln!("Failed to deliver packet with no fallback ({err})");
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{PacketLike, PacketSendListener};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    struct TestPacket {
        name: &'static str,
    }

    impl PacketLike for TestPacket {
        fn packet_type_name(&self) -> &'static str {
            self.name
        }
    }

    #[test]
    fn test_then_run_success() {
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();
        let cb = PacketSendListener::then_run(move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
        cb(Ok(()));
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_then_run_error_still_runs() {
        let count = Arc::new(AtomicUsize::new(0));
        let c = count.clone();
        let cb = PacketSendListener::then_run(move || {
            c.fetch_add(1, Ordering::SeqCst);
        });
        cb(Err("boom".to_string()));
        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_exceptionally_send_with_fallback() {
        let cb = PacketSendListener::exceptionally_send(|| {
            Some(TestPacket {
                name: "fallback_packet",
            })
        });
        cb(Err("send failed".to_string()));
        cb(Ok(()));
    }

    #[test]
    fn test_exceptionally_send_without_fallback() {
        let cb = PacketSendListener::exceptionally_send(|| Option::<TestPacket>::None);
        cb(Err("send failed".to_string()));
        cb(Ok(()));
    }
}
