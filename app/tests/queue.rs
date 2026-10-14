use localclip_app::queue::Queue;

#[test]
fn queue_replacement_busy_guard_and_idle_expiry() {
    let mut q = Queue::default();
    assert!(q.start(vec!["a".into()], 0).is_err());
    q.start(vec!["a".into(), "b".into()], 0).unwrap();
    let generation = q.generation;
    assert!(q.begin(100, true));
    assert!(!q.begin(100, true));
    assert_eq!(q.next_item(), Some((1, "a".into())));
    q.finish(generation);
    assert!(!q.expired(600_099));
    assert!(q.expired(600_100));
    q.start(vec!["c".into(), "d".into()], 600_100).unwrap();
    assert!(!q.begin(600_101, false));
    assert!(q.begin(600_101, true));
    q.finish(generation);
    assert!(q.busy);
    q.finish(q.generation);
    assert!(!q.busy);
    q.stop();
    assert!(!q.active());
}
