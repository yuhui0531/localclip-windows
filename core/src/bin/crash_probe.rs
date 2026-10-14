use localclip_core::{
    domain::{Item, Payload, Representation, Source},
    store::{HistoryStore, Options, Snapshot},
};
use std::{
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os().nth(1).ok_or("missing test database")?;
    let commits = Arc::new(AtomicUsize::new(0));
    let options = Options {
        available_space: Arc::new(|| Ok(u64::MAX)),
        before_commit: Arc::new(move || {
            if commits.fetch_add(1, Ordering::SeqCst) == 1 {
                println!("transaction-ready");
                std::io::stdout()
                    .flush()
                    .map_err(|_| localclip_core::store::StoreError::IoFailure)?;
                // Only a test process reaches this gate; the parent terminates it with an open transaction.
                loop {
                    std::thread::park();
                }
            }
            Ok(())
        }),
        ..Default::default()
    };
    let store = HistoryStore::open(path, options)?;
    for index in 0..2 {
        let payload = if index == 0 {
            Payload::text("synthetic-committed-baseline")
        } else {
            Payload {
                items: vec![Item {
                    representations: vec![Representation {
                        format: "image/png".into(),
                        data: vec![73; 4 * 1048576],
                    }],
                    ..Default::default()
                }],
            }
        };
        store.ingest(Snapshot {
            capture_id: format!("crash-{index}"),
            captured_at: 1760486400000 + index,
            payload,
            source: Source::default(),
            policy_revision: 0,
            sensitive: false,
        })?;
    }
    Ok(())
}
