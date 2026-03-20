//! Integration tests for action and requires attribute macros.

use marionette_macros::{action, requires};
use marionette_protocol::AuthRequirement;

#[action(name = "save-contact")]
async fn save_contact() {}

#[test]
fn action_macro_constant() {
    assert_eq!(SAVE_CONTACT, "save-contact");
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
