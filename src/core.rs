use hyper::body::Body;
use hyper::client::connect::HttpConnector;
use hyper::client::Client;
use hyper::header::HeaderValue;
use hyper::http::uri::{Authority, Scheme};
use hyper::HeaderMap;
use hyper::Request;
use hyper::Uri;
use std::ffi::OsString;
use tracing::{info, instrument};

pub struct Kaon {
    pub in_flight: bool,
    pub environment: std::env::VarsOs,
    pub client: Client<HttpConnector, Body>,
    pub runtime_api: Option<OsString>,
}

impl Kaon {
    #[instrument]
    async fn retrieve_environment() -> Option<OsString> {
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
                Some(_) => info!("| kaon environment | {:#?} is set", var),
                None => info!("| kaon environment | {:#?} is not set", var),
            }
        }

        // Some(aws_lambda_runtime_api)
        match std::env::var_os(aws_lambda_runtime_api) {
            Some(value) => Some(value),
            None => None,
        }
    }

    #[instrument]
    async fn build_uri(authority: Authority, path: &str) -> Result<hyper::Uri, hyper::http::Error> {
        let uri = Uri::builder()
            .scheme(Scheme::HTTP)
            .authority(authority)
            .path_and_query(format!("/2018-06-01{}", path))
            .build();
        match uri {
            Ok(built_uri) => {
                info!("| kaon uri | {:?} is built!", &built_uri);
                Ok(built_uri)
            }
            Err(error) => {
                info!("| kaon uri | {}", error);
                Err(error)
            }
        }
    }

    async fn get_event(&self) {
        let authority = self.runtime_api.as_ref().unwrap().to_str().unwrap();
        println!("{:?}", authority);

        let next_invocation = Uri::builder()
            .scheme("http")
            .authority(authority)
            .path_and_query("/runtime/invocation/next")
            .build()
            .unwrap();

        let response = &self.client.get(next_invocation).await;

        match &response {
            Ok(event) => {
                println!("{:?}", event.body());
                println!("{:?}", event.headers());
            }
            Err(error) => println!("{:?}", error),
        }
    }

    async fn tracing_header(header: &HeaderMap<HeaderValue>) {
        if header.contains_key("Lambda-Runtime-Trace-Id") {
            let x_amzn_trace_id = OsString::from("_X_AMZN_TRACE_ID");
            let value = &header
                .get("Lambda-Runtime-Trace-Id")
                .unwrap()
                .to_str()
                .unwrap();
            std::env::set_var(x_amzn_trace_id, OsString::from(&value));
        }
    }

    async fn handle_response(&self) {
        let request_id = String::from("156cb537-e2d4-11e8-9b34-d36013741fb9");
        let authority = self.runtime_api.as_ref().unwrap().to_str().unwrap();
        let path_and_query = format!("/runtime/invocation/{}/response", request_id);

        let invocation_response = Uri::builder()
            .scheme("http")
            .authority(authority)
            .path_and_query(path_and_query)
            .build()
            .unwrap();

        // let response = &self.client.post(next_invocation).await;
        let request = Request::builder()
            .method("POST")
            .uri(invocation_response)
            .body(Body::from("hi"))
            .unwrap();

        let response = &self.client.request(request).await;

        match &response {
            Ok(event) => {
                println!("{:?}", event.body());
                println!("{:?}", event.headers());
            }
            Err(error) => println!("{:?}", error),
        }
    }

    async fn invocation_error(&self) {
        let request_id = String::from("156cb537-e2d4-11e8-9b34-d36013741fb9");
        let authority = self.runtime_api.as_ref().unwrap().to_str().unwrap();
        let path_and_query = format!("/runtime/invocation/{}/error", request_id);

        let invocation_error = Uri::builder()
            .scheme("http")
            .authority(authority)
            .path_and_query(path_and_query)
            .build()
            .unwrap();

        let request = Request::builder()
            .method("POST")
            .header("Lambda-Runtime-Function-Error-Type", "Unhandled")
            .uri(invocation_error)
            .body(Body::from("some error"))
            .unwrap();

        let response = &self.client.request(request).await;

        match &response {
            Ok(event) => {
                println!("{:?}", event.body());
                println!("{:?}", event.headers());
            }
            Err(error) => println!("{:?}", error),
        }
    }

    async fn initialization_error(&self) {
        let authority = self.runtime_api.as_ref().unwrap().to_str().unwrap();
        // let path_and_query = format!("/runtime/init/error", request_id);

        let invocation_error = Uri::builder()
            .scheme("http")
            .authority(authority)
            .path_and_query("/runtime/init/error")
            .build()
            .unwrap();

        let request = Request::builder()
            .method("POST")
            .header("Lambda-Runtime-Function-Error-Type", "Unhandled")
            .uri(invocation_error)
            .body(Body::from("some error"))
            .unwrap();

        let response = &self.client.request(request).await;

        match &response {
            Ok(event) => {
                println!("{:?}", event.body());
                println!("{:?}", event.headers());
            }
            Err(error) => println!("{:?}", error),
        }
    }

    async fn process() {
        unimplemented!()
    }

    pub async fn charge() -> Kaon {
        // Self::retrieve_environment().await;

        Kaon {
            in_flight: false,
            environment: std::env::vars_os(),
            client: Client::new(),
            runtime_api: Self::retrieve_environment().await,
        }
    }

    pub async fn decay(&mut self) {
        self.in_flight = true;
        Self::process().await;
        Self::get_event(&self).await;
    }

    pub async fn stop(&mut self) {
        self.in_flight = false;
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
    async fn build_uri() -> Result<(), hyper::http::Error> {
        let authority = Authority::from_static("test_aws_lambda_runtime_api");
        let path = String::from("/runtime/invocation/next");
        let uri = Kaon::build_uri(authority, &path).await?;
        assert_eq!(uri.scheme(), Some(&Scheme::HTTP));
        assert_eq!(uri.host(), Some("test_aws_lambda_runtime_api"));
        assert_eq!(uri.path(), "/2018-06-01/runtime/invocation/next");
        Ok(())
    }

    #[tokio::test]
    async fn get_event() {
        let url = &mockito::server_address().to_string();
        std::env::set_var("AWS_LAMBDA_RUNTIME_API", url);
        let mock = mockito::mock("GET", "/runtime/invocation/next").create();
        let kaon = Kaon::charge().await;
        kaon.get_event().await;
        mock.assert();
        assert!(mock.matched());
    }

    #[tokio::test]
    async fn charge() {
        // std::env::set_var("AWS_LAMBDA_RUNTIME_API", "test_aws_lambda_runtime_api");
        let kaon = Kaon::charge().await;
        assert_eq!(kaon.in_flight, false);
        for (k, v) in kaon.environment {
            assert_eq!(std::env::var_os(k), Some(v));
        }
        assert_eq!(kaon.runtime_api.is_some(), false);
    }
}
