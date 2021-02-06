use hyper::body::Body;
use hyper::client::connect::HttpConnector;
use hyper::client::Client;
use hyper::header::HeaderValue;
use hyper::http::uri::Scheme;
use hyper::HeaderMap;
use hyper::Request;
use hyper::Response;
use hyper::Uri;
use std::ffi::OsString;
use tracing::{error, info, instrument};

#[derive(Debug)]
pub struct Api {
    pub client: Client<HttpConnector, Body>,
    pub runtime_api: String,
}

impl Api {
    #[instrument]
    async fn build_uri(authority: &str, path: &str) -> hyper::Uri {
        let uri = Uri::builder()
            .scheme(Scheme::HTTP)
            .authority(authority)
            .path_and_query(format!("/2018-06-01{}", path))
            .build();
        match uri {
            Ok(built_uri) => {
                info!("| kaon uri | Built URI - {:?}", &built_uri);
                built_uri
            }
            Err(error) => {
                error!("| kaon uri | {}", error);
                panic!("cannot build uri");
            }
        }
    }

    #[instrument]
    pub async fn set_tracing_header(header: &HeaderMap<HeaderValue>) {
        if header.contains_key("Lambda-Runtime-Trace-Id") {
            let x_amzn_trace_id = OsString::from("_X_AMZN_TRACE_ID");
            let value = &header
                .get("Lambda-Runtime-Trace-Id")
                .unwrap()
                .to_str()
                .unwrap();
            std::env::set_var(x_amzn_trace_id, OsString::from(&value));
        }
        info!("| kaon api | _X_AMZN_TRACE_ID environment variable set");
    }

    #[instrument]
    pub async fn runtime_next_invocation(&self) -> Result<Response<Body>, hyper::Error> {
        let path = "/runtime/invocation/next";
        let uri = Self::build_uri(&self.runtime_api, path).await;
        let response = self.client.get(uri).await;

        match response {
            Ok(event) => {
                info!("| kaon api | Event received!");
                Ok(event)
            }
            Err(error) => {
                error!("| kaon api | {:?}", error);
                Err(error)
            }
        }
        // Ok(())
    }

    #[instrument]
    pub async fn runtime_invocation_response(
        &self,
        request_id: &str,
        response: Body,
    ) -> Result<(), hyper::http::Error> {
        let path = format!("/runtime/invocation/{}/response", request_id);
        let uri = Self::build_uri(&self.runtime_api, &path).await;
        let request = Request::builder()
            .method("POST")
            .uri(uri)
            .body(response)
            .unwrap();
        let response = &self.client.request(request).await;

        match &response {
            Ok(event) => {
                info!("| kaon api | response {:?}", event.status());
            }
            Err(error) => error!("| kaon api | {:?}", error),
        }
        Ok(())
    }

    #[instrument]
    pub async fn runtime_invocation_error(
        &self,
        request_id: &str,
        error: Body,
    ) -> Result<(), hyper::http::Error> {
        let path = format!("/runtime/invocation/{}/error", request_id);
        let uri = Self::build_uri(&self.runtime_api, &path).await;
        let request = Request::builder()
            .method("POST")
            .header("Lambda-Runtime-Function-Error-Type", "Unhandled")
            .uri(uri)
            .body(error)
            .unwrap();
        let response = &self.client.request(request).await;

        match &response {
            Ok(event) => {
                info!("| kaon api | error sent {:?}", event.status());
            }
            Err(error) => error!("| kaon api | {:?}", error),
        }
        Ok(())
    }

    #[instrument]
    pub async fn runtime_initialization_error(
        &self,
        error: Body,
    ) -> Result<(), hyper::http::Error> {
        let path = "/runtime/init/error";
        let uri = Self::build_uri(&self.runtime_api, &path).await;
        let request = Request::builder()
            .method("POST")
            .header("Lambda-Runtime-Function-Error-Type", "Unhandled")
            .uri(uri)
            .body(error)
            .unwrap();
        let response = &self.client.request(request).await;

        match &response {
            Ok(event) => {
                info!("| kaon api | error sent {:?}", event.status())
            }
            Err(error) => error!("| kaon api | {:?}", error),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn build_uri() -> Result<(), hyper::http::Error> {
        let authority = "test_aws_lambda_runtime_api";
        let path = String::from("/runtime/invocation/next");
        let uri = Api::build_uri(authority, &path).await;
        assert_eq!(uri.scheme(), Some(&Scheme::HTTP));
        assert_eq!(uri.host(), Some("test_aws_lambda_runtime_api"));
        assert_eq!(uri.path(), "/2018-06-01/runtime/invocation/next");
        Ok(())
    }

    #[tokio::test]
    async fn set_tracing_header() {
        let mut test_headers = HeaderMap::new();
        let test_x_amzn_trace_id_header_key = "Lambda-Runtime-Trace-Id";
        let test_x_amzn_trace_id_header_value = HeaderValue::from_static(
            "Root=1-5759e988-bd862e3fe1be46a994272793;Parent=53995c3f42cd8ad8;Sampled=1",
        );
        assert_eq!(test_headers.len(), 0);
        test_headers.insert(
            test_x_amzn_trace_id_header_key,
            test_x_amzn_trace_id_header_value,
        );
        assert_eq!(test_headers.len(), 1);
        let test_environment_variable = OsString::from("_X_AMZN_TRACE_ID");
        assert_eq!(std::env::var_os(&test_environment_variable).is_none(), true);
        Api::set_tracing_header(&test_headers).await;
        assert_eq!(std::env::var_os(&test_environment_variable).is_some(), true);
        assert_eq!(
            std::env::var_os(test_environment_variable),
            Some(OsString::from(
                "Root=1-5759e988-bd862e3fe1be46a994272793;Parent=53995c3f42cd8ad8;Sampled=1"
            )),
        );
    }

    #[tokio::test]
    async fn runtime_next_invocation() -> Result<(), hyper::Error> {
        let test_api = Api {
            client: Client::new(),
            runtime_api: mockito::server_address().to_string(),
        };
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
            .create();
        let response = Api::runtime_next_invocation(&test_api).await?;
        mock.assert();
        assert!(mock.matched());
        assert!(&response.status().is_success());
        let headers = response.headers();
        assert_eq!(
            &headers.get("content-type").unwrap().to_str().unwrap(),
            &"application/json"
        );
        assert_eq!(
            &headers
                .get("Lambda-Runtime-Aws-Request-Id")
                .unwrap()
                .to_str()
                .unwrap(),
            &"8476a536-e9f4-11e8-9739-2dfe598c3fcd"
        );
        assert_eq!(
            &headers
                .get("Lambda-Runtime-Trace-Id")
                .unwrap()
                .to_str()
                .unwrap(),
            &"Root=1-5bef4de7-ad49b0e87f6ef6c87fc2e700;Parent=9a9197af755a6419;Sampled=1"
        );
        assert_eq!(
            &headers
                .get("Lambda-Runtime-Client-Context")
                .unwrap()
                .to_str()
                .unwrap(),
            &"test_aws_mobile_sdk_client_and_device"
        );
        assert_eq!(
            &headers
                .get("Lambda-Runtime-Cognito-Identity")
                .unwrap()
                .to_str()
                .unwrap(),
            &"test_aws_module_sdk_cognito_idp"
        );
        assert_eq!(
            &headers
                .get("Lambda-Runtime-Deadline-Ms")
                .unwrap()
                .to_str()
                .unwrap(),
            &"1542409706888"
        );
        assert_eq!(
            &headers
                .get("Lambda-Runtime-Invoked-Function-Arn")
                .unwrap()
                .to_str()
                .unwrap(),
            &"arn:aws:lambda:us-east-2:123456789012:function:custom-runtime"
        );
        let body = response.into_body();
        assert_eq!(
            hyper::body::to_bytes(body).await.unwrap(),
            r#"{"test": "kaon"}"#
        );
        Ok(())
    }

    #[tokio::test]
    async fn runtime_invocation_response() -> Result<(), hyper::http::Error> {
        let test_api = Api {
            client: Client::new(),
            runtime_api: mockito::server_address().to_string(),
        };
        let test_request_id = String::from("156cb537-e2d4-11e8-9b34-d36013741fb9");
        let test_body = Body::from("SUCCESS");
        let mock = mockito::mock(
            "POST",
            "/2018-06-01/runtime/invocation/156cb537-e2d4-11e8-9b34-d36013741fb9/response",
        )
        .match_body("SUCCESS")
        .create();
        Api::runtime_invocation_response(&test_api, &test_request_id, test_body).await?;
        mock.assert();
        assert!(mock.matched());
        Ok(())
    }

    #[tokio::test]
    async fn runtime_invocation_error() -> Result<(), hyper::http::Error> {
        let test_api = Api {
            client: Client::new(),
            runtime_api: mockito::server_address().to_string(),
        };
        let test_request_id = String::from("156cb537-e2d4-11e8-9b34-d36013741fb9");
        let test_error = Body::from(
            r#"{"errorMessage": "test_kaon_error_message", "errorType": "test_kaon_error_type"}"#,
        );
        let mock = mockito::mock(
            "POST",
            "/2018-06-01/runtime/invocation/156cb537-e2d4-11e8-9b34-d36013741fb9/error",
        )
        .match_body(
            r#"{"errorMessage": "test_kaon_error_message", "errorType": "test_kaon_error_type"}"#,
        )
        .create();
        Api::runtime_invocation_error(&test_api, &test_request_id, test_error).await?;
        mock.assert();
        assert!(mock.matched());
        Ok(())
    }

    #[tokio::test]
    async fn runtime_initialization_error() -> Result<(), hyper::http::Error> {
        let test_api = Api {
            client: Client::new(),
            runtime_api: mockito::server_address().to_string(),
        };
        let test_error = Body::from(
            r#"{"errorMessage": "test_kaon_error_message", "errorType": "test_kaon_error_type"}"#,
        );
        let mock = mockito::mock("POST", "/2018-06-01/runtime/init/error")
            .match_body(r#"{"errorMessage": "test_kaon_error_message", "errorType": "test_kaon_error_type"}"#)
            .create();
        Api::runtime_initialization_error(&test_api, test_error).await?;
        mock.assert();
        assert!(mock.matched());
        Ok(())
    }
}
