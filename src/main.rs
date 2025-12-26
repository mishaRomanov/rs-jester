use async_trait::async_trait;
use log::info;
use pingora::server::configuration::Opt;
use pingora::server::Server;
use pingora::services::background::background_service;
use pingora::upstreams::peer::HttpPeer;
use pingora::Result;
use pingora_proxy::{ProxyHttp, Session};
use std::{sync::Arc, time::Duration};

pub struct LB(
    Arc<pingora_load_balancing::LoadBalancer<pingora_load_balancing::selection::RoundRobin>>,
);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self
            .0
            .select(b"", 256) // hash doesn't matter
            .unwrap();

        info!("upstream peer is: {:?}", upstream);

        let peer = Box::new(HttpPeer::new(upstream, true, "one.one.one.one".to_string()));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut pingora_http::RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request
            .insert_header("Host", "one.one.one.one")
            .unwrap();
        Ok(())
    }
}

fn main() {
    // Command-line arguments.
    let opt = Opt::parse_args();
    // Building a server.
    let mut my_server = Server::new(Some(opt)).unwrap();
    // Setup.
    my_server.bootstrap();

    // 127.0.0.1:343" is just a bad server
    let mut upstreams = pingora_load_balancing::LoadBalancer::try_from_iter([
        "1.1.1.1:443",
        "1.0.0.1:443",
        "127.0.0.1:343",
    ])
    .unwrap();

    // We add health check in the background so that the bad server is never selected.
    let hc = pingora_load_balancing::health_check::TcpHealthCheck::new();
    upstreams.set_health_check(hc);
    upstreams.health_check_frequency = Some(Duration::from_secs(1));

    let background = background_service("health check", upstreams);

    let upstreams = background.task();

    let mut lb = pingora_proxy::http_proxy_service(&my_server.configuration, LB(upstreams));
    lb.add_tcp("0.0.0.0:6188");

    my_server.add_service(lb);
    my_server.add_service(background);
}
