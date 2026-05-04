//! Integration tests for action and requires attribute macros.

use marionette_macros::{action, requires};
use marionette_protocol::AuthRequirement;

#[action(name = "save-contact")]
async fn save_contact() {}

#[test]
fn action_macro_constant() {
    assert_eq!(SAVE_CONTACT, "save-contact");
}

// Action names commonly use `/` as a namespace separator
// (e.g. `"app/add-person"`); the macro must convert non-identifier
// characters to `_` so the const ident remains valid.
#[action(name = "app/add-person")]
async fn add_person() {}

#[test]
fn action_macro_namespaced_name_compiles() {
    assert_eq!(APP_ADD_PERSON, "app/add-person");
}

#[requires(authenticated)]
async fn edit_profile() {}

#[test]
fn requires_authenticated() {
    assert_eq!(EDIT_PROFILE_AUTH, AuthRequirement::Authenticated);
}

#[requires(role = "admin")]
async fn delete_user() {}

#[test]
fn requires_role() {
    assert_eq!(DELETE_USER_AUTH, AuthRequirement::Role("admin"));
}
