use hyper::body::Body;
use hyper::client::Client;
use std::ffi::OsString;
use tracing::{info, instrument, warn};

mod api;
mod context;

use crate::core::api::Api;
use crate::core::context::Context;

#[derive(Debug)]
pub struct Kaon {
    pub in_flight: bool,
    pub environment: std::env::VarsOs,
    pub api: Api,
    pub processed: Vec<Context>,
}

impl Kaon {
    #[instrument]
    async fn retrieve_environment() -> String {
        info!("| kaon environment | Checking environment variables");
        let handler = OsString::from("_HANDLER");
        let x_amzn_trace_id = OsString::from("_X_AMZN_TRACE_ID");
        let aws_region = OsString::from("AWS_REGION");
        let aws_execution_env = OsString::from("AWS_EXECUTION_ENV");
        let aws_lambda_function_name = OsString::from("AWS_LAMBDA_FUNCTION_NAME");
        let aws_lambda_function_memory_size = OsString::from("AWS_LAMBDA_FUNCTION_MEMORY_SIZE");
        let aws_lambda_function_version = OsString::from("AWS_LAMBDA_FUNCTION_VERSION");
        let aws_lambda_initialization_type = OsString::from("AWS_LAMBDA_INITIALIZATION_TYPE");
        let aws_lambda_log_group_name = OsString::from("AWS_LAMBDA_LOG_GROUP_NAME");
        let aws_lambda_log_stream_name = OsString::from("AWS_LAMBDA_LOG_STREAM_NAME");
        let aws_access_key_id = OsString::from("AWS_ACCESS_KEY_ID");
        let aws_secret_access_key = OsString::from("AWS_SECRET_ACCESS_KEY");
        let aws_session_token = OsString::from("AWS_SESSION_TOKEN");
        let aws_lambda_runtime_api = OsString::from("AWS_LAMBDA_RUNTIME_API");
        let lambda_task_root = OsString::from("LAMBDA_TASK_ROOT");
        let lamda_runtime_dir = OsString::from("LAMBDA_RUNTIME_DIR");
        let tz = OsString::from("TZ");

        let environment_variables = vec![
            handler,
            x_amzn_trace_id,
            aws_region,
            aws_execution_env,
            aws_lambda_function_name,
            aws_lambda_function_memory_size,
            aws_lambda_function_version,
            aws_lambda_initialization_type,
            aws_lambda_log_group_name,
            aws_lambda_log_stream_name,
            aws_lambda_runtime_api.clone(),
            lambda_task_root,
            lamda_runtime_dir,
            tz,
        ];

        let sensitive_environment_variables =
            vec![aws_access_key_id, aws_secret_access_key, aws_session_token];

        for var in environment_variables.iter() {
            match std::env::var_os(var) {
                Some(value) => info!("| kaon environment | {:#?}={:#?}", var, value),
                None => info!("| kaon environment | {:#?} is not found", var),
            }
        }

        for var in sensitive_environment_variables.iter() {
            match std::env::var_os(var) {
                Some(_) => warn!("| kaon environment | {:#?} is set", var),
                None => warn!("| kaon environment | {:#?} is not set", var),
            }
        }

        match std::env::var_os(&aws_lambda_runtime_api) {
            Some(value) => value.into_string().unwrap(),
            None => panic!(
                "| kaon environment| {:?} is not found - kaon cannot initialize!",
                &aws_lambda_runtime_api
            ),
        }
    }

    #[instrument]
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
    // async fn handler<F: Fn() + std::fmt::Debug>(f: F) {
    //     info!("| kaon handler | Kaon is invoking function!");
    //     f();
    // }
    async fn handler<Handler: Fn(Body, Context)>(handle: Handler, event: Body, context: Context) {
        handle(event, context);
    }

    #[instrument]
    pub async fn charge() -> Kaon {
        let api = Api {
            client: Client::new(),
            runtime_api: Self::retrieve_environment().await,
        };

        info!("| kaon charge | Kaon is charged!");

        Kaon {
            in_flight: false,
            environment: std::env::vars_os(),
            api,
            processed: Vec::with_capacity(20),
        }
    }

    // #[instrument]
    // pub async fn decay<
    //     Handler: Fn() + Copy + std::fmt::Display + std::fmt::Debug + std::ops::Fn(Body, Context) -> (),
    // >(
    //     &mut self,
    //     function: Handler,
    // ) {
    pub async fn decay<E, C>(&mut self, function: fn(E, C)) {
        self.in_flight = true;

        println!("{:?}", self.in_flight);

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
                while self.in_flight {
                    let fake_body = Body::from("more to come...");
                    let handle_response = self
                        .api
                        .runtime_invocation_response(context.aws_request_id.as_str(), fake_body)
                        .await;
                    if handle_response.is_ok() {
                        println!("event processed!");
                        break;
                    } else {
                        println!("handle response was not ok");
                        self.stop();
                    }
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
    async fn retrieve_settings() {
        std::env::set_var("RUST_LOG", "info");
        tracing_subscriber::fmt::init();

        let test_handler = OsString::from("_HANDLER");
        std::env::set_var(&test_handler, OsString::from("test_handler"));

        let test_x_amzn_trace_id = OsString::from("_X_AMZN_TRACE_ID");
        std::env::set_var(
            &test_x_amzn_trace_id,
            OsString::from("test_x_amzn_trace_id"),
        );

        let test_aws_region = OsString::from("AWS_REGION");
        std::env::set_var(&test_aws_region, OsString::from("test_aws_region"));

        let test_aws_exection_env = OsString::from("AWS_EXECUTION_ENV");
        std::env::set_var(
            &test_aws_exection_env,
            OsString::from("test_aws_exection_env"),
        );

        let test_aws_lambda_function_name = OsString::from("AWS_LAMBDA_FUNCTION_NAME");
        std::env::set_var(
            &test_aws_lambda_function_name,
            OsString::from("test_aws_lambda_function_name"),
        );

        let test_aws_lambda_function_memory_size =
            OsString::from("AWS_LAMBDA_FUNCTION_MEMORY_SIZE");
        std::env::set_var(
            &test_aws_lambda_function_memory_size,
            OsString::from("test_aws_lambda_function_memory_size"),
        );

        let test_aws_lambda_function_version = OsString::from("AWS_LAMBDA_FUNCTION_VERSION");
        std::env::set_var(
            &test_aws_lambda_function_version,
            OsString::from("test_aws_lambda_function_version"),
        );

        let test_aws_lambda_initialization_type = OsString::from("AWS_LAMBDA_INITIALIZATION_TYPE");
        std::env::set_var(
            &test_aws_lambda_initialization_type,
            OsString::from("test_aws_lambda_initialization_type"),
        );

        let test_aws_lambda_log_group_name = OsString::from("AWS_LAMBDA_LOG_GROUP_NAME");
        std::env::set_var(
            &test_aws_lambda_log_group_name,
            OsString::from("test_aws_lambda_log_group_name"),
        );

        let test_aws_lambda_log_stream_name = OsString::from("AWS_LAMBDA_LOG_STREAM_NAME");
        std::env::set_var(
            &test_aws_lambda_log_stream_name,
            OsString::from("test_aws_lambda_log_stream_name"),
        );

        let test_aws_access_key_id = OsString::from("AWS_ACCESS_KEY_ID");
        std::env::set_var(
            &test_aws_access_key_id,
            OsString::from("test_aws_access_key_id"),
        );

        let test_aws_secret_access_key = OsString::from("AWS_SECRET_ACCESS_KEY");
        std::env::set_var(
            &test_aws_secret_access_key,
            OsString::from("test_aws_secret_access_key"),
        );

        let test_aws_session_token = OsString::from("AWS_SESSION_TOKEN");
        std::env::set_var(
            &test_aws_session_token,
            OsString::from("test_aws_session_token"),
        );

        let test_aws_lambda_runtime_api = OsString::from("AWS_LAMBDA_RUNTIME_API");
        std::env::set_var(
            &test_aws_lambda_runtime_api,
            OsString::from("test_aws_lambda_runtime_api"),
        );

        let test_lambda_task_root = OsString::from("LAMBDA_TASK_ROOT");
        std::env::set_var(
            &test_lambda_task_root,
            OsString::from("test_lambda_task_root"),
        );

        let test_lambda_runtime_dir = OsString::from("LAMBDA_RUNTIME_DIR");
        std::env::set_var(
            &test_lambda_runtime_dir,
            OsString::from("test_lambda_runtime_dir"),
        );

        let test_tz = OsString::from("TZ");
        std::env::set_var(&test_tz, OsString::from("test_tz"));

        Kaon::retrieve_environment().await;

        let test_environment_variables = vec![
            test_handler,
            test_x_amzn_trace_id,
            test_aws_region,
            test_aws_exection_env,
            test_aws_lambda_function_name,
            test_aws_lambda_function_memory_size,
            test_aws_lambda_function_version,
            test_aws_lambda_initialization_type,
            test_aws_lambda_log_group_name,
            test_aws_lambda_log_stream_name,
            test_aws_access_key_id,
            test_aws_secret_access_key,
            test_aws_session_token,
            test_aws_lambda_runtime_api,
            test_lambda_task_root,
            test_lambda_runtime_dir,
            test_tz,
        ];

        for var in test_environment_variables.iter() {
            assert!(std::env::var_os(var).is_some());
        }
    }

    #[tokio::test]
    async fn handler() {
        // test currently does nothing
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
        let test_event = |_, _| println!("test kaon event!");
        let test_body = Body::from("test");
        Kaon::handler(test_event, test_body, test_context).await;
    }

    #[tokio::test]
    async fn charge() {
        std::env::set_var("AWS_LAMBDA_RUNTIME_API", "test_aws_lambda_runtime_api");
        let kaon = Kaon::charge().await;
        assert_eq!(kaon.in_flight, false);
        for (k, v) in kaon.environment {
            assert_eq!(std::env::var_os(k), Some(v));
        }
        assert_eq!(kaon.api.runtime_api.is_empty(), false);
    }

    #[tokio::test]
    async fn decay() {
        let test_aws_lambda_runtime_api = mockito::server_address().to_string();
        std::env::set_var("AWS_LAMBDA_RUNTIME_API", test_aws_lambda_runtime_api);
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
        let test_body = Body::from("test");
        let mock = mockito::mock("GET", "/2018-06-01/runtime/invocation/next")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header(
                "Lambda-Runtime-Aws-Request-Id",
                "8476a536-e9f4-11e8-9739-2dfe598c3fcd",
            )
            .with_header(
                "Lambda-Runtime-Trace-Id",
                "Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;Parent=9a9197af755a6419;Sampled=1",
            )
            .with_header(
                "Lambda-Runtime-Client-Context",
                "test_aws_mobile_sdk_client_and_device",
            )
            .with_header(
                "Lambda-Runtime-Cognito-Identity",
                "test_aws_module_sdk_cognito_idp",
            )
            .with_header("Lambda-Runtime-Deadline-Ms", "1542409706888")
            .with_header(
                "Lambda-Runtime-Invoked-Function-Arn",
                "arn:aws:lambda:us-east-2:123456789012:function:custom-runtime",
            )
            .with_body(r#"{"test": "kaon"}"#)
            .expect(2)
            .create();
        let mock_post = mockito::mock(
            "POST",
            "/2018-06-01/runtime/invocation/8476a536-e9f4-11e8-9739-2dfe598c3fcd/response",
        )
        .match_body("more to come...")
        .expect(1)
        .create();

        fn test_event(test_body: Body, test_context: Context) {
            println!("kaon test {:?}", test_body);
            println!("kaon test {}", test_context.aws_request_id);
        }

        let mut kaon = Kaon::charge().await;
        assert_eq!(kaon.in_flight, false);

        kaon.decay(test_event).await;
        mock.assert();
        assert!(mock.matched());
        mock_post.assert();
        assert!(mock_post.matched());
        kaon.stop();
        assert_eq!(kaon.in_flight, false);
    }
}
