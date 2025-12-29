use async_trait::async_trait;
use log::info;
use pingora_load_balancing::{selection::weighted::Weighted, LoadBalancer};
use std::sync::Arc;
use std::time;

use pingora::{
    server::configuration,
    services::{
        background::{background_service, BackgroundService, GenBackgroundService},
        Service,
    },
    upstreams::peer::HttpPeer,
    Result,
};
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};

pub struct Proxy(
    Arc<pingora_load_balancing::LoadBalancer<pingora_load_balancing::selection::RoundRobin>>,
);

#[async_trait]
impl ProxyHttp for Proxy {
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

impl Proxy {
    pub fn new_proxy_service(config: Arc<configuration::ServerConf>) -> impl Service {
        // Parse upstreams from somewhere (potentially database or static config)
        let mut balancer_upstreams = pingora_load_balancing::LoadBalancer::try_from_iter([
            "1.1.1.1:443",
            "1.0.0.1:443",
            "127.0.0.1:343",
        ])
        .unwrap();

        let hc = pingora_load_balancing::health_check::TcpHealthCheck::new();
        balancer_upstreams.set_health_check(hc);
        // TODO: parse from somewhere
        balancer_upstreams.health_check_frequency = Some(time::Duration::from_secs(1));

        let background = background_service("healthcheck", balancer_upstreams);
        // background.task() returns upstreams back.
        let upstreams = background.task();

        let mut balancer = http_proxy_service(&config, Proxy(upstreams));
        // Add a TCP listening endpoint with the given address (e.g., `127.0.0.1:8000`).
        balancer.add_tcp("0.0.0.0:6188");

        balancer
    }
}
