use pingora::server::configuration::Opt;
use pingora::server::Server;
mod balancer;

fn main() {
    // Command-line arguments.
    let opts = Opt::parse_args();
    // Building a server.
    let mut server = Server::new(Some(opts)).unwrap();

    // Creating balancer / proxy from config.
    let proxy_result = balancer::Proxy::new_proxy_service(server.configuration.clone());

    server.add_service(proxy_result);
    // Setup, according to docs.
    server.bootstrap();
    server.run_forever();
}
