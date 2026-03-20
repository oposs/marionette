use marionette_protocol::common::AuthRequirement;

use crate::error::ActionError;
use crate::extractors::Session;

/// Check whether the current session satisfies the given authorization requirement.
///
/// # Errors
///
/// Returns `ActionError::Unauthorized` if the session does not meet the requirement.
pub fn check_auth(requirement: &AuthRequirement, session: &Session) -> Result<(), ActionError> {
    match requirement {
        AuthRequirement::None => Ok(()),
        AuthRequirement::Authenticated => {
            if session.user_id.is_some() {
                Ok(())
            } else {
                Err(ActionError::Unauthorized(
                    "Authentication required".into(),
                ))
            }
        }
        AuthRequirement::Role(role) => {
            if session.user_id.is_none() {
                return Err(ActionError::Unauthorized(
                    "Authentication required".into(),
                ));
            }
            if session.roles.iter().any(|r| r == role) {
                Ok(())
            } else {
                Err(ActionError::Unauthorized(format!(
                    "Role '{role}' required"
                )))
            }
        }
    }
}
