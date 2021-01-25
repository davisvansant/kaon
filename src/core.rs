use hyper::body::Body;
use hyper::client::connect::HttpConnector;
use hyper::client::Client;
use std::ffi::OsString;
use tracing::{info, instrument};

pub struct Kaon {
    pub environment: std::env::VarsOs,
    pub client: Client<HttpConnector, Body>,
}

impl Kaon {
    #[instrument]
    async fn retrieve_environment() {
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
            aws_lambda_runtime_api,
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
    }

    async fn process() {
        unimplemented!()
    }

    pub async fn charge() -> Kaon {
        Self::retrieve_environment().await;

        Kaon {
            environment: std::env::vars_os(),
            client: Client::new(),
        }
    }

    pub async fn decay() {
        Self::process().await;
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
    async fn charge() {
        let kaon = Kaon::charge().await;
        for (k, v) in kaon.environment {
            assert_eq!(std::env::var_os(k), Some(v));
        }
    }
}
