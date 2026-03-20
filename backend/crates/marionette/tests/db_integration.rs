use marionette::db::session;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[tokio::test]
async fn test_migration_runs() {
    let _db = marionette::test_db().await;
    // If we get here without panic, migrations ran successfully.
}

#[tokio::test]
async fn test_create_session() {
    let db = marionette::test_db().await;
    let model = session::ActiveModel {
        session_token: Set("tok-create-001".into()),
        session_roles: Set("[]".into()),
        session_created: Set("2026-01-01T00:00:00".into()),
        session_expires: Set("2026-01-02T00:00:00".into()),
        ..Default::default()
    };
    let result = model.insert(&db).await.unwrap();
    assert!(result.session_id > 0);
    assert_eq!(result.session_token, "tok-create-001");
    assert_eq!(result.session_roles, "[]");
}

#[tokio::test]
async fn test_find_session_by_token() {
    let db = marionette::test_db().await;
    let model = session::ActiveModel {
        session_token: Set("tok-find-001".into()),
        session_user: Set(Some(42)),
        session_roles: Set(r#"["admin"]"#.into()),
        session_created: Set("2026-01-01T00:00:00".into()),
        session_expires: Set("2026-01-02T00:00:00".into()),
        ..Default::default()
    };
    let inserted = model.insert(&db).await.unwrap();

    let found = session::Entity::find()
        .filter(session::Column::SessionToken.eq("tok-find-001"))
        .one(&db)
        .await
        .unwrap()
        .expect("session should be found");

    assert_eq!(found.session_id, inserted.session_id);
    assert_eq!(found.session_user, Some(42));
    assert_eq!(found.session_roles, r#"["admin"]"#);
}

#[tokio::test]
async fn test_update_session() {
    let db = marionette::test_db().await;
    let model = session::ActiveModel {
        session_token: Set("tok-update-001".into()),
        session_roles: Set("[]".into()),
        session_created: Set("2026-01-01T00:00:00".into()),
        session_expires: Set("2026-01-02T00:00:00".into()),
        ..Default::default()
    };
    let inserted = model.insert(&db).await.unwrap();
    assert_eq!(inserted.session_user, None);

    let mut active: session::ActiveModel = inserted.into();
    active.session_user = Set(Some(99));
    let updated = active.update(&db).await.unwrap();
    assert_eq!(updated.session_user, Some(99));

    // Verify persistence
    let found = session::Entity::find()
        .filter(session::Column::SessionToken.eq("tok-update-001"))
        .one(&db)
        .await
        .unwrap()
        .expect("session should exist");
    assert_eq!(found.session_user, Some(99));
}

#[tokio::test]
async fn test_delete_session() {
    let db = marionette::test_db().await;
    let model = session::ActiveModel {
        session_token: Set("tok-delete-001".into()),
        session_roles: Set("[]".into()),
        session_created: Set("2026-01-01T00:00:00".into()),
        session_expires: Set("2026-01-02T00:00:00".into()),
        ..Default::default()
    };
    let inserted = model.insert(&db).await.unwrap();
    let id = inserted.session_id;

    // Delete
    sea_orm::ModelTrait::delete(inserted, &db).await.unwrap();

    // Verify gone
    let found = session::Entity::find_by_id(id).one(&db).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_multiple_sessions() {
    let db = marionette::test_db().await;
    for i in 0..3 {
        let model = session::ActiveModel {
            session_token: Set(format!("tok-multi-{i:03}")),
            session_roles: Set("[]".into()),
            session_created: Set("2026-01-01T00:00:00".into()),
            session_expires: Set("2026-01-02T00:00:00".into()),
            ..Default::default()
        };
        model.insert(&db).await.unwrap();
    }
    let all = session::Entity::find().all(&db).await.unwrap();
    assert_eq!(all.len(), 3);
}
