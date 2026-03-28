use orc_core::pty::{PtyError, PtyEvent, PtyEventHandler, PtyManager};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct MockHandler {
    events: Arc<Mutex<Vec<(String, MockEvent)>>>,
}

#[derive(Debug)]
enum MockEvent {
    Data,
    Exit,
}

impl PtyEventHandler for MockHandler {
    fn on_event(&self, session_id: &str, event: PtyEvent) {
        let mock = match event {
            PtyEvent::Data(_) => MockEvent::Data,
            PtyEvent::Exit => MockEvent::Exit,
        };
        self.events
            .lock()
            .unwrap()
            .push((session_id.to_string(), mock));
    }
}

fn create_manager() -> (PtyManager, Arc<Mutex<Vec<(String, MockEvent)>>>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let handler = Arc::new(MockHandler {
        events: Arc::clone(&events),
    });
    (PtyManager::new(handler), events)
}

fn wait_for_events(
    events: &Arc<Mutex<Vec<(String, MockEvent)>>>,
    min_count: usize,
    timeout: Duration,
) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if events.lock().unwrap().len() >= min_count {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    events.lock().unwrap().len() >= min_count
}

#[test]
fn test_spawn_and_receive_data() {
    let (manager, events) = create_manager();
    manager.spawn("s1".to_string(), 24, 80).unwrap();

    assert!(
        wait_for_events(&events, 1, Duration::from_secs(5)),
        "expected at least one data event from shell"
    );

    let evts = events.lock().unwrap();
    let has_data = evts
        .iter()
        .any(|(id, e)| id == "s1" && matches!(e, MockEvent::Data));
    assert!(has_data, "expected Data event for session s1");

    drop(evts);
    manager.kill("s1").unwrap();
}

#[test]
fn test_spawn_and_receive_exit() {
    let (manager, events) = create_manager();
    manager.spawn("s2".to_string(), 24, 80).unwrap();

    // send exit command
    manager.write("s2", b"exit\n").unwrap();

    assert!(
        wait_for_events(&events, 2, Duration::from_secs(5)),
        "expected events after exit command"
    );

    // wait a bit more for exit event
    std::thread::sleep(Duration::from_millis(500));

    let evts = events.lock().unwrap();
    let has_exit = evts
        .iter()
        .any(|(id, e)| id == "s2" && matches!(e, MockEvent::Exit));
    assert!(has_exit, "expected Exit event for session s2");
}

#[test]
fn test_write_to_session() {
    let (manager, events) = create_manager();
    manager.spawn("s3".to_string(), 24, 80).unwrap();

    // wait for initial shell output
    wait_for_events(&events, 1, Duration::from_secs(3));

    let count_before = events.lock().unwrap().len();

    // write echo command
    manager.write("s3", b"echo hello_pty_test\n").unwrap();

    // wait for echo response
    std::thread::sleep(Duration::from_millis(500));

    let count_after = events.lock().unwrap().len();
    assert!(
        count_after > count_before,
        "expected new data events after write"
    );

    manager.kill("s3").unwrap();
}

#[test]
fn test_kill_session() {
    let (manager, _events) = create_manager();
    manager.spawn("s4".to_string(), 24, 80).unwrap();
    manager.kill("s4").unwrap();

    // after kill, write should fail with SessionNotFound
    let result = manager.write("s4", b"test");
    assert!(matches!(result, Err(PtyError::SessionNotFound(_))));
}

#[test]
fn test_write_nonexistent_session() {
    let (manager, _events) = create_manager();
    let result = manager.write("nonexistent", b"test");
    assert!(matches!(result, Err(PtyError::SessionNotFound(_))));
}

#[test]
fn test_resize() {
    let (manager, _events) = create_manager();
    manager.spawn("s5".to_string(), 24, 80).unwrap();

    let result = manager.resize("s5", 48, 120);
    assert!(result.is_ok(), "resize should succeed");

    manager.kill("s5").unwrap();
}
