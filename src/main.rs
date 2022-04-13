use std::net::TcpListener;
use zero2prod::configurations::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    println!("Listening on {}", address);
    let listener = TcpListener::bind(address).expect("Cannot bind address");
    run(listener)?.await
}
