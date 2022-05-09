use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use std::time::Duration;
use zero2prod::configurations::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let conn_string = configuration.database.connection_string();
    let pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_lazy(&conn_string.expose_secret())
        .expect("Failed to connect to DB");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    tracing::info!("Listening on {}", address);
    let listener = TcpListener::bind(address).expect("Cannot bind address");
    run(listener, pool)?.await
}
