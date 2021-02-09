use crate::core::Context;
use std::future::Future;
use std::io::{Error, ErrorKind};

pub async fn handler<E, R, Outatime, Handler: Fn(E, Context) -> Outatime>(
    handle: Handler,
    event: E,
    context: Context,
) -> Result<R, std::io::Error>
where
    Handler: Fn(E, Context) -> Outatime,
    Outatime: Future<Output = Result<R, std::io::Error>>,
{
    let event_result = handle(event, context).await;

    match event_result {
        Ok(result) => Ok(result),
        Err(error) => Err(Error::new(ErrorKind::Other, error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn run_handler() -> Result<(), std::io::Error> {
        struct TestRequest {
            test_request: String,
        }

        struct TestResponse {
            test_response: String,
            test_context: Context,
        }

        let test_request = TestRequest {
            test_request: String::from("hello"),
        };

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

        async fn test_handler_function(
            event: TestRequest,
            context: Context,
        ) -> Result<TestResponse, std::io::Error> {
            let response = TestResponse {
                test_response: event.test_request,
                test_context: context,
            };
            Ok(response)
        }

        let result = handler(test_handler_function, test_request, test_context).await;

        if let Ok(test_kaon_result) = result {
            assert_eq!(test_kaon_result.test_response, String::from("hello"));
            assert_eq!(
                test_kaon_result.test_context.aws_request_id,
                String::from("8476a536-e9f4-11e8-9739-2dfe598c3fcd"),
            );
            assert_eq!(
                test_kaon_result.test_context.invoked_function_arn,
                String::from("arn:aws:lambda:us-east-2:123456789012:function:custom-runtime"),
            );
            assert_eq!(
                test_kaon_result.test_context.identity,
                String::from("test_identity"),
            );
            assert_eq!(
                test_kaon_result.test_context.client_context,
                String::from("test_client_context"),
            );
        }
        Ok(())
    }
}
