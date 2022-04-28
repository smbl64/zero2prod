use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configurations::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

// To make sure the logging is initialized once
static TRACING: Lazy<()> = Lazy::new(|| {
    let logger_name = "zero2prod".into();
    let log_level = "info".into();

    // We cannot use a variable for `subscriber`, because `get_subscriber` returns
    // different types.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(logger_name, log_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(logger_name, log_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    address: String,
    db_pool: PgPool,
}

async fn config_database(config: &DatabaseSettings) -> PgPool {
    // Create a new temporary database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to the DB");

    let query = format!(r#"CREATE DATABASE "{}";"#, config.database_name);
    connection
        .execute(query.as_str())
        .await
        .expect("Failed to create a temporary DB.");

    // Migrate
    let conn_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to DB.");

    sqlx::migrate!("./migrations")
        .run(&conn_pool)
        .await
        .expect("Failed to run migrations");

    conn_pool
}

async fn spawn_app() -> TestApp {
    // The first time this is called, the code in TRACING will be called.
    // Subsequent runs will ignore the code in TRACING.
    Lazy::force(&TRACING);

    let mut config = get_configuration().expect("Failed to read the config");
    // A temporary database to run our test
    config.database.database_name = Uuid::new_v4().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a random port");
    let port = listener.local_addr().unwrap().port();

    let db_pool = config_database(&config.database).await;

    let server =
        zero2prod::startup::run(listener, db_pool.clone()).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("Failed to execute the request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=jason%20bourne&email=jason_bourne%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute the request");
    assert_eq!(response.status().as_u16(), 200);

    let record = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(record.name, "jason bourne");
    assert_eq!(record.email, "jason_bourne@gmail.com");
}

#[tokio::test]
async fn subscribe_fails_with_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=jason%20bourne", "missing the email"),
        ("email=jason_bourne%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error_msg) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", test_app.address))
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
