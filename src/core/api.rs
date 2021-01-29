use hyper::body::Body;
use hyper::client::connect::HttpConnector;
use hyper::client::Client;
// use hyper::header::HeaderValue;
use hyper::http::uri::Scheme;
// use hyper::HeaderMap;
use hyper::Request;
use hyper::Uri;
// use std::ffi::OsString;
use tracing::{info, instrument};

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
                info!("| kaon uri | {:?} is built!", &built_uri);
                built_uri
            }
            Err(error) => {
                info!("| kaon uri | {}", error);
                panic!("cannot build uri");
            }
        }
    }

    pub async fn runtime_next_event(&self) -> Result<(), hyper::http::Error> {
        let path = "/runtime/invocation/next";
        let uri = Self::build_uri(&self.runtime_api, path).await;
        let response = &self.client.get(uri).await;

        match &response {
            Ok(event) => {
                println!("{:?}", event.body());
                println!("{:?}", event.headers());
            }
            Err(error) => println!("{:?}", error),
        }
        Ok(())
    }

    // async fn tracing_header(header: &HeaderMap<HeaderValue>) {
    //     if header.contains_key("Lambda-Runtime-Trace-Id") {
    //         let x_amzn_trace_id = OsString::from("_X_AMZN_TRACE_ID");
    //         let value = &header
    //             .get("Lambda-Runtime-Trace-Id")
    //             .unwrap()
    //             .to_str()
    //             .unwrap();
    //         std::env::set_var(x_amzn_trace_id, OsString::from(&value));
    //     }
    // }

    pub async fn runtime_invocation_response(&self) {
        let request_id = String::from("156cb537-e2d4-11e8-9b34-d36013741fb9");
        let path = format!("/runtime/invocation/{}/response", request_id);
        let uri = Self::build_uri(&self.runtime_api, &path).await;
        let request = Request::builder()
            .method("POST")
            .uri(uri)
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

    pub async fn runtime_invocation_error(&self) {
        let request_id = String::from("156cb537-e2d4-11e8-9b34-d36013741fb9");
        let path = format!("/runtime/invocation/{}/error", request_id);
        let uri = Self::build_uri(&self.runtime_api, &path).await;
        let request = Request::builder()
            .method("POST")
            .header("Lambda-Runtime-Function-Error-Type", "Unhandled")
            .uri(uri)
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

    pub async fn runtime_initialization_error(&self) {
        let path = "/runtime/init/error";
        let uri = Self::build_uri(&self.runtime_api, &path).await;
        let request = Request::builder()
            .method("POST")
            .header("Lambda-Runtime-Function-Error-Type", "Unhandled")
            .uri(uri)
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
    async fn runtime_next_event() -> Result<(), hyper::http::Error> {
        let test_api = Api {
            client: Client::new(),
            runtime_api: mockito::server_address().to_string(),
        };
        let mock = mockito::mock("GET", "/2018-06-01/runtime/invocation/next").create();
        Api::runtime_next_event(&test_api).await?;
        mock.assert();
        assert!(mock.matched());
        Ok(())
    }
}
