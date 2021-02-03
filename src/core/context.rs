pub struct Context {
    pub aws_request_id: String,
    pub invoked_function_arn: String,
    pub identity: String,
    pub client_context: String,
}

impl Context {
    pub async fn create(
        aws_request_id: String,
        invoked_function_arn: String,
        identity: String,
        client_context: String,
    ) -> Context {
        Context {
            aws_request_id,
            invoked_function_arn,
            identity,
            client_context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn context() {
        let test_aws_request_id = String::from("8476a536-e9f4-11e8-9739-2dfe598c3fcd");
        let test_arn =
            String::from("arn:aws:lambda:us-east-2:123456789012:function:custom-runtime");
        let test_identity = String::from("test_identity");
        let test_client_context = String::from("test_client_context");

        let test_context = Context::create(
            test_aws_request_id,
            test_arn,
            test_identity,
            test_client_context,
        )
        .await;
        assert_eq!(
            test_context.aws_request_id,
            String::from("8476a536-e9f4-11e8-9739-2dfe598c3fcd")
        );
        assert_eq!(
            test_context.invoked_function_arn,
            String::from("arn:aws:lambda:us-east-2:123456789012:function:custom-runtime")
        );
        assert_eq!(test_context.identity, String::from("test_identity"));
        assert_eq!(
            test_context.client_context,
            String::from("test_client_context")
        );
    }
}
