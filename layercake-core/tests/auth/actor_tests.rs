use layercake::auth::{Actor, SystemActor};

#[test]
fn actor_tracks_roles_and_scopes() {
    let actor = Actor::user(42)
        .with_role("viewer")
        .with_scope("read:project");

    assert_eq!(actor.user_id, Some(42));
    assert!(actor.has_role("viewer"));
    assert!(actor.has_scope("read:project"));
}

#[test]
fn system_actor_is_system() {
    let actor = SystemActor::internal();
    assert!(actor.is_system());
    assert!(actor.user_id.is_none());
}
