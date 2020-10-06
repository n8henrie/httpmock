extern crate httpmock;

use httpmock::Method::POST;
use httpmock::{Mock, MockServer};
use httpmock_macros::httpmock_example_test;
use isahc::prelude::*;
use serde_json::{json, Value};

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn json_value_body_test() {
    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let m = Mock::new()
        .expect_method(POST)
        .expect_path("/users")
        .expect_header("Content-Type", "application/json")
        .expect_json_body(json!({ "name": "Fred" }))
        .return_status(201)
        .return_header("Content-Type", "application/json")
        .return_json_body(json!({ "name": "Hans" }))
        .create_on(&server);

    // Act: Send the request and deserialize the response to JSON
    let mut response = Request::post(&format!("http://{}/users", server.address()))
        .header("Content-Type", "application/json")
        .body(json!({ "name": "Fred" }).to_string())
        .unwrap()
        .send()
        .unwrap();

    let user: Value =
        serde_json::from_str(&response.text().unwrap()).expect("cannot deserialize JSON");

    // Assert
    assert_eq!(response.status(), 201);
    assert_eq!(m.hits(), 1);
    assert_eq!(user.as_object().unwrap().get("name").unwrap(), "Hans");
}

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn json_body_object_serde_test() {
    // This is a temporary type that we will use for this test
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestUser {
        name: String,
    }

    // Arrange
    let _ = env_logger::try_init();
    let server = MockServer::start();

    let m = Mock::new()
        .expect_method(POST)
        .expect_path("/users")
        .expect_header("Content-Type", "application/json")
        .expect_json_body_obj(&TestUser {
            name: String::from("Fred"),
        })
        .return_status(201)
        .return_header("Content-Type", "application/json")
        .return_json_body_obj(&TestUser {
            name: String::from("Hans"),
        })
        .create_on(&server);

    // Act: Send the request and deserialize the response to JSON
    let mut response = Request::post(&format!("http://{}/users", server.address()))
        .header("Content-Type", "application/json")
        .body(
            json!(&TestUser {
                name: "Fred".to_string()
            })
            .to_string(),
        )
        .unwrap()
        .send()
        .unwrap();

    let user: TestUser =
        serde_json::from_str(&response.text().unwrap()).expect("cannot deserialize JSON");

    // Assert
    assert_eq!(response.status(), 201);
    assert_eq!(user.name, "Hans");
    assert_eq!(m.hits(), 1);
}

#[test]
#[httpmock_example_test] // Internal macro to make testing easier. Ignore it.
fn partial_json_body_test() {
    let _ = env_logger::try_init();
    let server = MockServer::start();

    // This is the structure that needs to be included in the request
    #[derive(serde::Serialize, serde::Deserialize)]
    struct ChildStructure {
        some_attribute: String,
    }

    // This is a parent structure that carries the included structure
    #[derive(serde::Serialize, serde::Deserialize)]
    struct ParentStructure {
        some_other_value: String,
        child: ChildStructure,
    }

    // Arranging the test by creating HTTP mocks.
    let m = Mock::new()
        .expect_method(POST)
        .expect_path("/users")
        .expect_json_body_partial(
            r#"
            {
                "child" : {
                    "some_attribute" : "Fred"
                }
            }
        "#,
        )
        .return_status(201)
        .return_body(r#"{"result":"success"}"#)
        .create_on(&server);

    // Simulates application that makes the request to the mock.
    let uri = format!("http://{}/users", m.server_address());
    let response = Request::post(&uri)
        .header("Content-Type", "application/json")
        .header("User-Agent", "rust-test")
        .body(
            serde_json::to_string(&ParentStructure {
                child: ChildStructure {
                    some_attribute: "Fred".to_string(),
                },
                some_other_value: "Flintstone".to_string(),
            })
            .unwrap(),
        )
        .unwrap()
        .send()
        .unwrap();

    // Assertions
    assert_eq!(response.status(), 201);
    assert_eq!(m.hits(), 1);
}
