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

#[cfg(test)]
mod tests {
    use super::*;

    fn anonymous_session() -> Session {
        Session {
            user_id: None,
            roles: vec![],
        }
    }

    fn authenticated_session(user_id: &str) -> Session {
        Session {
            user_id: Some(user_id.into()),
            roles: vec![],
        }
    }

    fn session_with_roles(user_id: &str, roles: &[&str]) -> Session {
        Session {
            user_id: Some(user_id.into()),
            roles: roles.iter().map(|r| (*r).to_string()).collect(),
        }
    }

    #[test]
    fn auth_none_passes_anonymous() {
        let result = check_auth(&AuthRequirement::None, &anonymous_session());
        assert!(result.is_ok());
    }

    #[test]
    fn auth_none_passes_authenticated() {
        let result = check_auth(&AuthRequirement::None, &authenticated_session("u1"));
        assert!(result.is_ok());
    }

    #[test]
    fn auth_authenticated_passes_with_user() {
        let result = check_auth(
            &AuthRequirement::Authenticated,
            &authenticated_session("u1"),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn auth_authenticated_rejects_anonymous() {
        let result = check_auth(&AuthRequirement::Authenticated, &anonymous_session());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ActionError::Unauthorized(_)));
    }

    #[test]
    fn auth_role_passes_with_matching_role() {
        let session = session_with_roles("u1", &["admin", "user"]);
        let result = check_auth(&AuthRequirement::Role("admin"), &session);
        assert!(result.is_ok());
    }

    #[test]
    fn auth_role_rejects_wrong_role() {
        let session = session_with_roles("u1", &["user"]);
        let result = check_auth(&AuthRequirement::Role("admin"), &session);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ActionError::Unauthorized(_)));
    }

    #[test]
    fn auth_role_rejects_anonymous() {
        let result = check_auth(&AuthRequirement::Role("admin"), &anonymous_session());
        assert!(result.is_err());
    }
}
