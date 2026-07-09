//! Concurrency and behaviour tests for the collaboration actor model
//! (CollaborationCoordinator + ProjectActor).
//!
//! These were previously untested (Horizon 1, Stage 5.3). They drive the public
//! `CoordinatorHandle` API and assert presence broadcasting, cursor/document
//! updates, dead-connection cleanup, empty-project removal, and correctness
//! under concurrent joins.

use layercake_server::collaboration::CollaborationCoordinator;
use layercake_server::server::websocket::types::{CursorPosition, DocumentType, ServerMessage};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

/// A fake client connection: the mpsc receiver the actor sends ServerMessages to.
struct FakeClient {
    rx: mpsc::Receiver<ServerMessage>,
}

impl FakeClient {
    fn new() -> (mpsc::Sender<ServerMessage>, Self) {
        let (tx, rx) = mpsc::channel(64);
        (tx, Self { rx })
    }

    /// Await the next message, failing the test if none arrives promptly.
    async fn next(&mut self) -> ServerMessage {
        timeout(Duration::from_secs(2), self.rx.recv())
            .await
            .expect("timed out waiting for a server message")
            .expect("connection closed unexpectedly")
    }

    /// Drain all currently-buffered messages without blocking.
    fn drain(&mut self) -> Vec<ServerMessage> {
        let mut out = Vec::new();
        while let Ok(msg) = self.rx.try_recv() {
            out.push(msg);
        }
        out
    }
}

fn canvas_pos(x: f64, y: f64) -> CursorPosition {
    CursorPosition::Canvas {
        x,
        y,
        zoom: Some(1.0),
    }
}

#[tokio::test]
async fn join_sends_bulk_presence_to_new_user_and_notifies_existing() {
    let coord = CollaborationCoordinator::spawn();

    // First user joins an empty project.
    let (tx_a, mut client_a) = FakeClient::new();
    coord
        .join_project(1, "a".into(), "Alice".into(), None, tx_a)
        .await
        .unwrap();

    // Alice's first message is her own bulk presence snapshot.
    match client_a.next().await {
        ServerMessage::BulkPresence { data } => {
            assert_eq!(data.len(), 1);
            assert_eq!(data[0].user_id, "a");
        }
        other => panic!("expected BulkPresence, got {:?}", other),
    }

    // Second user joins; Bob gets his own bulk presence (2 users), Alice gets a
    // UserPresence notification about Bob.
    let (tx_b, mut client_b) = FakeClient::new();
    coord
        .join_project(1, "b".into(), "Bob".into(), None, tx_b)
        .await
        .unwrap();

    match client_b.next().await {
        ServerMessage::BulkPresence { data } => {
            assert_eq!(data.len(), 2, "Bob sees both users");
        }
        other => panic!("expected BulkPresence, got {:?}", other),
    }

    match client_a.next().await {
        ServerMessage::UserPresence { data } => {
            assert_eq!(data.user_id, "b");
            assert!(data.is_online);
        }
        other => panic!("expected UserPresence for Bob, got {:?}", other),
    }

    coord.shutdown().await;
}

#[tokio::test]
async fn cursor_update_is_broadcast_to_other_users_in_document() {
    let coord = CollaborationCoordinator::spawn();

    let (tx_a, mut client_a) = FakeClient::new();
    let (tx_b, mut client_b) = FakeClient::new();
    coord
        .join_project(7, "a".into(), "Alice".into(), None, tx_a)
        .await
        .unwrap();
    coord
        .join_project(7, "b".into(), "Bob".into(), None, tx_b)
        .await
        .unwrap();

    // Both users switch into the same document.
    coord
        .switch_document(7, "a".into(), "doc1".into(), DocumentType::Canvas)
        .await;
    coord
        .switch_document(7, "b".into(), "doc1".into(), DocumentType::Canvas)
        .await;

    // update_cursor / switch_document are fire-and-forget. Round-trip a health
    // request (which the actor processes in order after those commands) so all
    // switch chatter is emitted before we drain it.
    let _ = coord.get_project_health(7).await;
    let _ = client_a.drain();
    let _ = client_b.drain();

    // Alice moves her cursor; Bob (same document) should receive a
    // DocumentActivity update, Alice should not receive her own.
    coord
        .update_cursor(7, "a".into(), "doc1".into(), canvas_pos(10.0, 20.0), None)
        .await;

    match client_b.next().await {
        ServerMessage::DocumentActivity { data } => {
            assert_eq!(data.document_id, "doc1");
            assert!(data.active_users.iter().any(|u| u.user_id == "a"));
        }
        other => panic!("expected DocumentActivity, got {:?}", other),
    }

    // Round-trip again so any (incorrect) self-echo would have been delivered,
    // then assert Alice received none for her own cursor move.
    let _ = coord.get_project_health(7).await;
    let self_msgs = client_a.drain();
    assert!(
        !self_msgs
            .iter()
            .any(|m| matches!(m, ServerMessage::DocumentActivity { .. })),
        "cursor owner should not receive its own cursor echo"
    );

    coord.shutdown().await;
}

#[tokio::test]
async fn leave_removes_user_and_empty_project_is_dropped() {
    let coord = CollaborationCoordinator::spawn();

    let (tx_a, _client_a) = FakeClient::new();
    coord
        .join_project(3, "a".into(), "Alice".into(), None, tx_a)
        .await
        .unwrap();

    let health = coord.get_project_health(3).await;
    assert_eq!(health.active_users, 1);
    assert_eq!(health.active_connections, 1);

    coord.leave_project(3, "a".into()).await.unwrap();

    // Project is empty, so it is removed from the coordinator; health reports
    // "not found" (zero everything).
    let health = coord.get_project_health(3).await;
    assert_eq!(health.active_users, 0);
    assert_eq!(health.active_connections, 0);

    // Leaving an already-removed project is reported as not found, not a panic.
    let err = coord.leave_project(3, "a".into()).await;
    assert!(err.is_err());

    coord.shutdown().await;
}

#[tokio::test]
async fn dead_connection_is_cleaned_up_on_next_broadcast() {
    let coord = CollaborationCoordinator::spawn();

    // Alice's receiver is dropped immediately => her connection is dead.
    let (tx_dead, dead_client) = FakeClient::new();
    coord
        .join_project(9, "a".into(), "Alice".into(), None, tx_dead)
        .await
        .unwrap();
    drop(dead_client);

    // Bob joins; broadcasting his presence to Alice fails, so Alice's dead
    // connection is cleaned up. After that only Bob remains.
    let (tx_b, _client_b) = FakeClient::new();
    coord
        .join_project(9, "b".into(), "Bob".into(), None, tx_b)
        .await
        .unwrap();

    // Give the actor a moment to process the failed send + cleanup.
    // (Poll health until Alice is gone rather than sleeping a fixed time.)
    let mut users = usize::MAX;
    for _ in 0..50 {
        users = coord.get_project_health(9).await.active_users;
        if users == 1 {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(
        users, 1,
        "dead connection (Alice) cleaned up, only Bob remains"
    );

    coord.shutdown().await;
}

#[tokio::test]
async fn concurrent_joins_all_register() {
    let coord = CollaborationCoordinator::spawn();

    // 25 users join the same project concurrently. The single-actor design must
    // serialise these without losing any.
    let mut handles = Vec::new();
    let mut clients = Vec::new();
    for i in 0..25 {
        let (tx, client) = FakeClient::new();
        clients.push(client);
        let coord = coord.clone();
        handles.push(tokio::spawn(async move {
            coord
                .join_project(5, format!("u{}", i), format!("User {}", i), None, tx)
                .await
        }));
    }

    for h in handles {
        h.await.unwrap().expect("join should succeed");
    }

    let health = coord.get_project_health(5).await;
    assert_eq!(health.active_users, 25, "all concurrent joins registered");
    assert_eq!(health.active_connections, 25);

    coord.shutdown().await;
}

#[tokio::test]
async fn switch_document_broadcasts_activity() {
    let coord = CollaborationCoordinator::spawn();

    let (tx_a, mut client_a) = FakeClient::new();
    let (tx_b, mut client_b) = FakeClient::new();
    coord
        .join_project(2, "a".into(), "Alice".into(), None, tx_a)
        .await
        .unwrap();
    coord
        .join_project(2, "b".into(), "Bob".into(), None, tx_b)
        .await
        .unwrap();
    let _ = client_a.drain();
    let _ = client_b.drain();

    // Alice switches into a document; both members in that document get a
    // DocumentActivity broadcast.
    coord
        .switch_document(2, "a".into(), "doc-x".into(), DocumentType::Spreadsheet)
        .await;

    match client_a.next().await {
        ServerMessage::DocumentActivity { data } => {
            assert_eq!(data.document_id, "doc-x");
            assert!(data.active_users.iter().any(|u| u.user_id == "a"));
        }
        other => panic!("expected DocumentActivity, got {:?}", other),
    }

    coord.shutdown().await;
}
