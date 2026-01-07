use pingora::server::configuration::Opt;
use pingora::server::Server;
mod balancer;
mod config;

fn main() {
    // Command-line arguments.
    let opts = Opt::parse_args();
    // Building a server.
    let mut server = Server::new(Some(opts)).unwrap();

    // Creating balancer / proxy from config.
    let proxy = balancer::Proxy::new_proxy_service(server.configuration.clone());

    server.add_service(proxy);
    // Setup, according to docs.
    server.bootstrap();
    server.run_forever();
}
