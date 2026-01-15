use pingora::server::configuration::Opt;
use pingora::server::Server;
mod balancer;
mod config;
mod metrics;

fn main() {
    // Command-line arguments.
    let opts = Opt::parse_args();

    // Maybe use additional cli args for that.
    // Init logger.
    tracing::subscriber::set_global_default(tracing_subscriber::fmt().finish())
        .expect("failed to initialize tracing subscriber");

    // Try to build a server.
    match Server::new(Some(opts)) {
        Ok(mut server) => {
            // Creating balancer / proxy from config.
            let proxy = balancer::Proxy::new_proxy_service(server.configuration.clone());

            server.add_service(proxy);
            // Setup, according to docs.
            server.bootstrap();
            server.run_forever();
        }
        Err(e) => {
            panic!("Failed to build and start the server: {}", e);
        }
    }
}
