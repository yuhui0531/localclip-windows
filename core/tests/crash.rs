use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    sync::mpsc,
    time::Duration,
};

#[test]
fn fifty_forced_terminations_preserve_atomic_records() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/crash-tests")
        .join(std::process::id().to_string());
    assert!(!root.exists());
    std::fs::create_dir_all(&root).unwrap();
    for iteration in 0..50 {
        let path = root.join(format!("crash-{iteration}.sqlite"));
        let mut child = Command::new(env!("CARGO_BIN_EXE_crash_probe"))
            .arg(&path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            let mut line = String::new();
            let result = BufReader::new(stdout).read_line(&mut line);
            let _ = tx.send((result, line));
        });
        let ready = rx.recv_timeout(Duration::from_secs(30));
        #[cfg(target_os = "windows")]
        {
            let status = Command::new("taskkill")
                .args(["/F", "/PID", &child.id().to_string()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            assert!(status.success());
        }
        #[cfg(target_os = "macos")]
        child.kill().unwrap();
        child.wait().unwrap();
        reader.join().unwrap();
        let (read, line) = ready.unwrap();
        read.unwrap();
        assert_eq!(line.trim(), "transaction-ready");
        let db = rusqlite::Connection::open(&path).unwrap();
        assert_eq!(
            db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok",
            "iteration {iteration}"
        );
        for table in ["clips", "items", "representations", "blobs", "text_content"] {
            assert_eq!(
                db.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                1,
                "iteration {iteration}, {table}"
            );
        }
        assert_eq!(
            db.query_row("SELECT storeRevision FROM store_meta", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            db.query_row("SELECT payload FROM blobs", [], |r| r.get::<_, Vec<u8>>(0))
                .unwrap(),
            b"synthetic-committed-baseline"
        );
        let foreign_keys: i64 = db
            .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(foreign_keys, 0);
    }
    std::fs::remove_dir_all(root).unwrap();
    println!("forced-termination iterations=50 integrity=50 atomic-records=50 foreign-keys=50");
}
