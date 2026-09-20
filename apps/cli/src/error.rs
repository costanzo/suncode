use suncode_sdk::BusinessError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub code: String,
    pub message: String,
    pub exit_code: u8,
}

impl CliError {
    pub fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_arguments".into(),
            message: message.into(),
            exit_code: 2,
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self {
            code: "io_error".into(),
            message: message.into(),
            exit_code: 1,
        }
    }

    pub fn from_business(error: BusinessError) -> Self {
        let exit_code = match error.code.as_str() {
            "invalid_arguments" | "serialization_error" => 2,
            "model_unavailable" | "credential_unavailable" | "provider_not_found" => 3,
            "approval_required"
            | "question_required"
            | "authorization_denied"
            | "policy_denied" => 4,
            "agent_already_active" => 5,
            _ => 1,
        };
        Self {
            code: error.code,
            message: error.message,
            exit_code,
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_stable_exit_statuses() {
        assert_eq!(
            CliError::from_business(BusinessError::new("agent_already_active", "busy")).exit_code,
            5
        );
        assert_eq!(
            CliError::from_business(BusinessError::new("model_unavailable", "missing")).exit_code,
            3
        );
        assert_eq!(
            CliError::from_business(BusinessError::new("database_error", "failed")).exit_code,
            1
        );
    }
}
