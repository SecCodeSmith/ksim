use ksim_core::event::EventQueue;
use ksim_core::time::SimTime;

#[test]
fn pop_due_returns_none_before_event_time() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(10), 42);
    let result = queue.pop_due(SimTime::from_secs(5));
    assert_eq!(result, None);
}

#[test]
fn pop_due_returns_event_at_exact_time() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(10), 42);
    let result = queue.pop_due(SimTime::from_secs(10));
    assert_eq!(result, Some((SimTime::from_secs(10), 42)));
}

#[test]
fn pop_due_returns_event_after_time() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(10), 42);
    let result = queue.pop_due(SimTime::from_secs(15));
    assert_eq!(result, Some((SimTime::from_secs(10), 42)));
}

#[test]
fn pop_due_earliest_first() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(20), 2);
    queue.push(SimTime::from_secs(10), 1);
    queue.push(SimTime::from_secs(30), 3);

    let result1 = queue.pop_due(SimTime::from_secs(100));
    assert_eq!(result1, Some((SimTime::from_secs(10), 1)));

    let result2 = queue.pop_due(SimTime::from_secs(100));
    assert_eq!(result2, Some((SimTime::from_secs(20), 2)));

    let result3 = queue.pop_due(SimTime::from_secs(100));
    assert_eq!(result3, Some((SimTime::from_secs(30), 3)));
}

#[test]
fn pop_due_fifo_for_equal_times() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(10), 1);
    queue.push(SimTime::from_secs(10), 2);
    queue.push(SimTime::from_secs(10), 3);

    let result1 = queue.pop_due(SimTime::from_secs(10));
    assert_eq!(result1, Some((SimTime::from_secs(10), 1)));

    let result2 = queue.pop_due(SimTime::from_secs(10));
    assert_eq!(result2, Some((SimTime::from_secs(10), 2)));

    let result3 = queue.pop_due(SimTime::from_secs(10));
    assert_eq!(result3, Some((SimTime::from_secs(10), 3)));
}

#[test]
fn next_time_returns_none_for_empty_queue() {
    let queue = EventQueue::<i32>::new();
    assert_eq!(queue.next_time(), None);
}

#[test]
fn next_time_returns_earliest_event_time() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(20), 2);
    queue.push(SimTime::from_secs(10), 1);
    queue.push(SimTime::from_secs(30), 3);

    assert_eq!(queue.next_time(), Some(SimTime::from_secs(10)));
}

#[test]
fn next_time_unchanged_after_failed_pop() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(20), 2);

    let _ = queue.pop_due(SimTime::from_secs(10));
    assert_eq!(queue.next_time(), Some(SimTime::from_secs(20)));
}

#[test]
fn len_for_empty_queue() {
    let queue = EventQueue::<i32>::new();
    assert_eq!(queue.len(), 0);
}

#[test]
fn len_counts_all_events() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(10), 1);
    queue.push(SimTime::from_secs(20), 2);
    queue.push(SimTime::from_secs(30), 3);

    assert_eq!(queue.len(), 3);
}

#[test]
fn len_decreases_after_pop() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(10), 1);
    queue.push(SimTime::from_secs(20), 2);

    assert_eq!(queue.len(), 2);
    let _ = queue.pop_due(SimTime::from_secs(100));
    assert_eq!(queue.len(), 1);
}

#[test]
fn is_empty_for_new_queue() {
    let queue = EventQueue::<i32>::new();
    assert!(queue.is_empty());
}

#[test]
fn is_empty_false_after_push() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(10), 1);
    assert!(!queue.is_empty());
}

#[test]
fn is_empty_true_after_pop_all() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(10), 1);

    let _ = queue.pop_due(SimTime::from_secs(100));
    assert!(queue.is_empty());
}

#[test]
fn pop_due_never_returns_events_after_now() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(5), 1);
    queue.push(SimTime::from_secs(10), 2);
    queue.push(SimTime::from_secs(15), 3);

    let result1 = queue.pop_due(SimTime::from_secs(8));
    assert_eq!(result1, Some((SimTime::from_secs(5), 1)));

    let result2 = queue.pop_due(SimTime::from_secs(8));
    assert_eq!(result2, None);
}

#[test]
fn pop_due_respects_time_boundary() {
    let mut queue = EventQueue::<i32>::new();
    queue.push(SimTime::from_secs(10), 1);
    queue.push(SimTime::from_secs(11), 2);

    let result = queue.pop_due(SimTime::from_secs(10));
    assert_eq!(result, Some((SimTime::from_secs(10), 1)));

    let result2 = queue.pop_due(SimTime::from_secs(10));
    assert_eq!(result2, None);
}

#[test]
fn mixed_scenario_with_duplicates_and_ordering() {
    let mut queue = EventQueue::<String>::new();
    queue.push(SimTime::from_secs(30), "c".to_string());
    queue.push(SimTime::from_secs(10), "a1".to_string());
    queue.push(SimTime::from_secs(10), "a2".to_string());
    queue.push(SimTime::from_secs(20), "b".to_string());

    let (t1, e1) = queue.pop_due(SimTime::from_secs(100)).unwrap();
    assert_eq!(t1, SimTime::from_secs(10));
    assert_eq!(e1, "a1");

    let (t2, e2) = queue.pop_due(SimTime::from_secs(100)).unwrap();
    assert_eq!(t2, SimTime::from_secs(10));
    assert_eq!(e2, "a2");

    let (t3, e3) = queue.pop_due(SimTime::from_secs(100)).unwrap();
    assert_eq!(t3, SimTime::from_secs(20));
    assert_eq!(e3, "b");
}

#[test]
fn empty_queue_pop_due() {
    let mut queue = EventQueue::<i32>::new();
    assert_eq!(queue.pop_due(SimTime::from_secs(100)), None);
}
