use std::env;

#[test]
fn privileged_loader_smoke() {
    if env::var("DRISHTI_PRIVILEGED_TESTS").ok().as_deref() != Some("1") {
        eprintln!("skipping privileged smoke test; set DRISHTI_PRIVILEGED_TESTS=1 to enable");
        return;
    }

    #[cfg(feature = "ebpf-runtime")]
    {
        use drishti_daemon::{config::Config, loader};
        use tokio::runtime::Runtime;
        use tokio::sync::{mpsc, watch};

        let runtime = Runtime::new().expect("tokio runtime should initialize");
        runtime.block_on(async {
            let config = Config::default();
            let (tx, _rx) = mpsc::channel(64);
            let (_shutdown_tx, shutdown_rx) = watch::channel(false);

            let started = loader::start(config, tx, shutdown_rx, false).await;
            assert!(
                started.is_ok(),
                "loader should initialize with eBPF runtime"
            );
        });
    }

    #[cfg(not(feature = "ebpf-runtime"))]
    {
        eprintln!(
            "skipping privileged smoke test because drishti-daemon was not built with ebpf-runtime feature"
        );
    }
}
