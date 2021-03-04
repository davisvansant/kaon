use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub enum ErrorType {
    Unhandled,
}

#[derive(Deserialize, Serialize)]
pub struct ErrorRequest {
    error_message: String,
    error_type: ErrorType,
    stack_trace: String,
}

impl ErrorRequest {
    pub async fn collect(error_message: String) -> ErrorRequest {
        ErrorRequest {
            error_message,
            error_type: ErrorType::Unhandled,
            stack_trace: String::from("unused"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn collect() {
        let test_error = String::from("some test error");
        let test_error_request = ErrorRequest::collect(test_error).await;
        assert_eq!(
            test_error_request.error_message,
            String::from("some test error"),
        );
        assert_eq!(test_error_request.error_type, ErrorType::Unhandled);
        assert_eq!(test_error_request.stack_trace, String::from("unused"));
    }
}
