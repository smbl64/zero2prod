use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configurations::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let conn_string = configuration.database.connection_string();
    let pool = PgPool::connect(&conn_string)
        .await
        .expect("Failed to connect to DB.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    println!("Listening on {}", address);
    let listener = TcpListener::bind(address).expect("Cannot bind address");
    run(listener, pool)?.await
}
