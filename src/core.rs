use hyper::body::Body;
use hyper::client::Client;
// use std::ffi::OsString;
use serde::{Deserialize, Serialize};
use std::future::Future;
use tracing::{info, instrument, warn};

mod api;
mod context;
mod handler;
mod initialization_tasks;

use crate::core::api::Api;
use crate::core::context::Context;
use crate::core::handler::EventHandler;
use crate::core::initialization_tasks::retrieve_settings;

#[derive(Debug)]
pub struct Kaon {
    pub in_flight: bool,
    pub environment: std::env::VarsOs,
    pub api: Api,
    pub processed: Vec<Context>,
}

impl Kaon {
    // #[instrument]
    async fn collect_event(&mut self, new_event: Context) {
        let event = self.processed.last();
        if let Some(last_event) = event {
            if last_event.aws_request_id.as_str() == new_event.aws_request_id.as_str() {
                self.stop();
                info!("| kaon collect event | event has already been processed!");
            } else {
                self.processed.push(new_event);
                info!("| kaon collect event | event processed!");
            }
        } else {
            self.processed.push(new_event);
            info!("| kaon collect event | event processed!");
        }
    }

    // #[instrument]
    pub async fn charge() -> Kaon {
        let api = Api {
            client: Client::new(),
            runtime_api: retrieve_settings().await,
        };

        Self {
            in_flight: false,
            environment: std::env::vars_os(),
            api,
            processed: Vec::with_capacity(20),
        }
    }

    pub async fn decay<F, B, C, D>(&mut self, function: F)
    where
        B: for<'de> Deserialize<'de> + Copy,
        C: Serialize + Copy,
        F: Fn(B, Context) -> D,
        D: Future<Output = Result<C, ()>> + Serialize,
    {
        self.in_flight = true;

        let handler = EventHandler::init(function).await;

        // info!("| kaon decay | Kaon decay is in process ...");

        while self.in_flight {
            let event = self.api.runtime_next_invocation().await;

            if let Ok(event_response) = event {
                let headers = event_response.headers();
                Api::set_tracing_header(headers).await;
                let id = &headers.get("Lambda-Runtime-Aws-Request-Id").unwrap();
                let arn = &headers.get("Lambda-Runtime-Invoked-Function-Arn").unwrap();
                let identity = &headers.get("Lambda-Runtime-Cognito-Identity").unwrap();
                let client = &headers.get("Lambda-Runtime-Client-Context").unwrap();
                let context = Context::create(
                    id.to_str().unwrap().to_string(),
                    arn.to_str().unwrap().to_string(),
                    identity.to_str().unwrap().to_string(),
                    client.to_str().unwrap().to_string(),
                )
                .await;
                self.collect_event(context.clone()).await;
                // checkpoint to see if we want to continue processing
                let cloned_body = event_response.into_body();
                let response_body = hyper::body::to_bytes(cloned_body).await.unwrap();
                let response_json = serde_json::from_slice(&response_body).unwrap();
                while self.in_flight {
                    // let fake_body = Body::from("more to come...");
                    let handler_result = handler.run(response_json, context.clone()).await;
                    match handler_result {
                        Ok(result) => {
                            let handler_json_response = serde_json::to_vec(&result).unwrap();
                            let response_body = Body::from(handler_json_response);
                            let handle_response = self
                                .api
                                .runtime_invocation_response(
                                    context.aws_request_id.as_str(),
                                    response_body,
                                )
                                .await;
                            if handle_response.is_ok() {
                                println!("event processed!");
                                break;
                            } else {
                                println!("handle response was not ok");
                                self.stop();
                            }
                        }
                        Err(_) => panic!("better error goes here"),
                    };
                    // let handler_json_response = serde_json::to_vec(&handler_result).unwrap();
                    // let handle_response = self
                    //     .api
                    //     .runtime_invocation_response(
                    //         context.aws_request_id.as_str(),
                    //         handler_json_response,
                    //     )
                    //     .await;
                    // if handle_response.is_ok() {
                    //     println!("event processed!");
                    //     break;
                    // } else {
                    //     println!("handle response was not ok");
                    //     self.stop();
                    // }
                }
            } else {
                println!("error connecting to api");
                self.stop();
            }
        }
    }

    pub fn stop(&mut self) {
        self.in_flight = false;
        info!("| kaon decay | Kaon decay stopped ...");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn charge() {
        std::env::set_var("AWS_LAMBDA_RUNTIME_API", "test_aws_lambda_runtime_api");
        // struct TestRequest {
        //     test_request: String,
        // }
        //
        // struct TestResponse {
        //     pub test_response: String,
        //     pub test_context: Context,
        // }

        // let test_request = TestRequest {
        //     test_request: String::from("hello"),
        // };
        //
        // let test_aws_request_id = String::from("8476a536-e9f4-11e8-9739-2dfe598c3fcd");
        // let test_arn =
        //     String::from("arn:aws:lambda:us-east-2:123456789012:function:custom-runtime");
        // let test_identity = String::from("test_identity");
        // let test_client_context = String::from("test_client_context");
        //
        // let test_context = Context::create(
        //     test_aws_request_id,
        //     test_arn,
        //     test_identity,
        //     test_client_context,
        // )
        // .await;

        // async fn test_handler_function(
        //     event: TestRequest,
        //     context: Context,
        // ) -> Result<TestResponse, std::io::Error> {
        //     let response = TestResponse {
        //         test_response: event.test_request,
        //         test_context: context,
        //     };
        //     Ok(response)
        // }
        let kaon = Kaon::charge().await;
        assert_eq!(kaon.in_flight, false);
        for (k, v) in kaon.environment {
            assert_eq!(std::env::var_os(k), Some(v));
        }
        assert_eq!(kaon.api.runtime_api.is_empty(), false);
        assert_eq!(kaon.processed.is_empty(), true);
    }

    // #[tokio::test]
    // async fn decay() {
    //     let test_aws_lambda_runtime_api = mockito::server_address().to_string();
    //     std::env::set_var("AWS_LAMBDA_RUNTIME_API", test_aws_lambda_runtime_api);
    //     let test_aws_request_id = String::from("8476a536-e9f4-11e8-9739-2dfe598c3fcd");
    //     let test_arn =
    //         String::from("arn:aws:lambda:us-east-2:123456789012:function:custom-runtime");
    //     let test_identity = String::from("test_identity");
    //     let test_client_context = String::from("test_client_context");
    //
    //     let test_context = Context::create(
    //         test_aws_request_id,
    //         test_arn,
    //         test_identity,
    //         test_client_context,
    //     )
    //     .await;
    //     let test_body = Body::from("test");
    //     let mock = mockito::mock("GET", "/2018-06-01/runtime/invocation/next")
    //         .with_status(200)
    //         .with_header("content-type", "application/json")
    //         .with_header(
    //             "Lambda-Runtime-Aws-Request-Id",
    //             "8476a536-e9f4-11e8-9739-2dfe598c3fcd",
    //         )
    //         .with_header(
    //             "Lambda-Runtime-Trace-Id",
    //             "Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;Parent=9a9197af755a6419;Sampled=1",
    //         )
    //         .with_header(
    //             "Lambda-Runtime-Client-Context",
    //             "test_aws_mobile_sdk_client_and_device",
    //         )
    //         .with_header(
    //             "Lambda-Runtime-Cognito-Identity",
    //             "test_aws_module_sdk_cognito_idp",
    //         )
    //         .with_header("Lambda-Runtime-Deadline-Ms", "1542409706888")
    //         .with_header(
    //             "Lambda-Runtime-Invoked-Function-Arn",
    //             "arn:aws:lambda:us-east-2:123456789012:function:custom-runtime",
    //         )
    //         .with_body(r#"{"test": "kaon"}"#)
    //         .expect(2)
    //         .create();
    //     let mock_post = mockito::mock(
    //         "POST",
    //         "/2018-06-01/runtime/invocation/8476a536-e9f4-11e8-9739-2dfe598c3fcd/response",
    //     )
    //     .match_body("more to come...")
    //     .expect(1)
    //     .create();
    //
    //     fn test_event(test_body: Body, test_context: Context) {
    //         println!("kaon test {:?}", test_body);
    //         println!("kaon test {}", test_context.aws_request_id);
    //     }
    //
    //     let mut kaon = Kaon::charge().await;
    //     assert_eq!(kaon.in_flight, false);
    //
    //     kaon.decay(test_event).await;
    //     mock.assert();
    //     assert!(mock.matched());
    //     mock_post.assert();
    //     assert!(mock_post.matched());
    //     kaon.stop();
    //     assert_eq!(kaon.in_flight, false);
    // }
}
