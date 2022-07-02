use std::net::TcpListener;
use zero2prod::run;
// Lunch our app in the background somehow lol
// No .await call, therefore no need for `spawn_app` to be async now.
// We are also running tests, so it is not worth it to propagate errors:
// If we fail to perform the required setup we can just panic and crash all things.
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Fail to bind address");
    // We now launch the server as a background task
    // tokio::spawn returns a handle of the spawned feature
    // but we have no use for it hence the non binding let
    let _ = tokio::spawn(server);

    // We return the application address to the caller
    format!("http://127.0.0.1:{}", port)
}

// `tokio::test` is the equivalent of `tokio::main`.
// It also spares you from having to specify the `#[test] attribute.
//
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<- Name of the test file)
#[tokio::test]
async fn health_check_works() {
    // Arrange
    // ! Removed await and expect
    let address = spawn_app();

    // We need to bring in "reqwest"
    // To perform HTTP Request against our application
    let client = reqwest::Client::new();

    // Act
    let response = client
        // Use the returned application address
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_return_200_for_valid_form_data() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16())
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
