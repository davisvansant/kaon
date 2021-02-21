use crate::core::Context;
// use serde::{Deserialize, Serialize};
use std::future::Future;

#[derive(Debug)]
pub struct EventHandler<EventFunction> {
    function: EventFunction,
}

impl<EventFunction> EventHandler<EventFunction> {
    pub async fn init<EventRequest, EventResponse, Outatime>(
        function: EventFunction,
    ) -> EventHandler<EventFunction>
    where
        EventFunction: Fn(EventRequest, Context) -> Outatime,
        Outatime: Future<Output = Result<EventResponse, ()>>,
    {
        EventHandler { function }
    }

    pub async fn run<EventRequest, EventResponse, Outatime>(
        &self,
        event: EventRequest,
        context: Context,
    ) -> Result<EventResponse, ()>
    where
        EventFunction: Fn(EventRequest, Context) -> Outatime,
        Outatime: Future<Output = Result<EventResponse, ()>>,
    {
        let event_result = (self.function)(event, context).await;

        match event_result {
            Ok(result) => Ok(result),
            Err(error) => {
                println!("{:?}", error);
                Err(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn init() {
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
        ) -> Result<TestResponse, ()> {
            let response = TestResponse {
                test_response: event.test_request,
                test_context: context,
            };
            Ok(response)
        }

        let event_handler = EventHandler::init(test_handler_function).await;
        let test_result = event_handler.run(test_request, test_context).await;
        if let Ok(event_result) = test_result {
            assert_eq!(event_result.test_response, String::from("hello"));
            assert_eq!(
                event_result.test_context.aws_request_id,
                String::from("8476a536-e9f4-11e8-9739-2dfe598c3fcd"),
            );
            assert_eq!(
                event_result.test_context.invoked_function_arn,
                String::from("arn:aws:lambda:us-east-2:123456789012:function:custom-runtime"),
            );
            assert_eq!(
                event_result.test_context.identity,
                String::from("test_identity"),
            );
            assert_eq!(
                event_result.test_context.client_context,
                String::from("test_client_context"),
            );
        }
    }
}
