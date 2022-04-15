use sqlx::{Connection, PgConnection};
use std::net::TcpListener;
use zero2prod::configurations::get_configuration;

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to address");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::startup::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app();
    let config = get_configuration().expect("Failed to read configurations.");
    let conn_string = config.database.connection_string();
    //
    // The `Connection` trait MUST be in scope for us to invoke
    // `PgConnection::connect`, it is not an inherent method of the struct!
    let mut connection = PgConnection::connect(&conn_string)
        .await
        .expect("Failed to connect to DB.");

    let client = reqwest::Client::new();

    let body = "name=jason%20bourne&email=jason_bourne%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute the request");
    assert_eq!(response.status().as_u16(), 200);

    let record = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(record.name, "jason bourne");
    assert_eq!(record.email, "jason_bourne@gmail.com");
}

#[tokio::test]
async fn subscribe_fails_with_400_when_data_is_missing() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=jason%20bourne", "missing the email"),
        ("email=jason_bourne%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error_msg) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute the request");
        assert_eq!(
            response.status().as_u16(),
            400,
            "The api did not return 400 when the payload was: {}",
            error_msg
        );
    }
}
