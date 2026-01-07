use crate::config;
use async_trait::async_trait;
use pingora::{
    server::configuration,
    services::{background::background_service, Service},
    upstreams::peer::HttpPeer,
    Result,
};
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};
use std::sync::Arc;
use std::time;
use uuid;

pub struct Proxy(
    Arc<pingora_load_balancing::LoadBalancer<pingora_load_balancing::selection::RoundRobin>>,
);

// Define const to beautify code
const HOST_HEADER_NAME: &str = "target.backend";

#[async_trait]
impl ProxyHttp for Proxy {
    // По сути, вся основная логика балансировки здесь.
    // Надо изучить все методы интерфейса ProxyHttp и понять, какие из них можно переопределить для
    // реализации нужной логики.
    // Нужно понять, че я ваще хочу сделать.

    // TODO: мб какую-нибудь простую структурку контекста сделать? наверное кстати можно в
    // контекст запихать метрики или информацию по апстримам
    type CTX = RequestContext;

    // Here we create a new context for each request. We generate uuid for each request.
    // This should ease potential troubleshooting and logging.
    fn new_ctx(&self) -> Self::CTX {
        RequestContext {
            req_id: uuid::Uuid::new_v4(),
        }
    }

    // Method responsible for selecting an upstream peer for the given session.
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // Select upstream from balancer.
        if let Some(upstream) = self.0.select(b"", 256) {
            // Create a peer from the selected upstream.
            // httppeer::new() takes (upstream, use_tls, server_name (SNI))

            tracing::info!("Redirecting request to {}", &upstream.addr);
            let peer = Box::new(HttpPeer::new(upstream, false, HOST_HEADER_NAME.to_string()));

            Ok(peer)
        } else {
            tracing::error!("Failed to select an upstream peer: no healthy upstreams available");
            Err(pingora::Error::new(pingora::Custom(
                "failed to select an upstream",
            )))
        }
    }

    // Pre-process the upstream request before sending it to the upstream server.
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut pingora_http::RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request.insert_header("Host", HOST_HEADER_NAME)?;
        upstream_request.insert_header("X-Request-ID", ctx.req_id.to_string().as_str())?;

        Ok(())
    }

    // TODO: self.request_filter
    // TODO: self.response_filter для метрик

    // TODO: self.fail_to_proxy()
}

impl Proxy {
    // Constructor for the proxy service.
    pub fn new_proxy_service(config: Arc<configuration::ServerConf>) -> impl Service {
        //TODO: переписать конструктор на from_backends() и распарсить бекенды вместе с весами
        // Структура ниже:
        // pub struct Backend {
        //     /// The address to the backend server.
        //     pub addr: SocketAddr,
        //     /// The relative weight of the server. Load balancing algorithms will
        //     /// proportionally distributed traffic according to this value.
        //     pub weight: usize,
        //
        //     /// The extension field to put arbitrary data to annotate the Backend.
        //     /// The data added here is opaque to this crate hence the data is ignored by
        //     /// functionalities of this crate. For example, two backends with the same
        //     /// [SocketAddr] and the same weight but different `ext` data are considered
        //     /// identical.
        //     /// See [Extensions] for how to add and read the data.
        //     #[derivative(PartialEq = "ignore")]
        //     #[derivative(PartialOrd = "ignore")]
        //     #[derivative(Hash = "ignore")]
        //     #[derivative(Ord = "ignore")]
        //     pub ext: Extensions,
        // }

        // Parsing proxy configuration from environment variables, config files, etc
        let proxy_config = config::ProxyConfig::new();

        // Parse upstreams from somewhere (potentially database or static config)
        let mut balancer_upstreams =
            pingora_load_balancing::LoadBalancer::try_from_iter(["127.0.0.1:8080"]).unwrap();

        let hc = pingora_load_balancing::health_check::TcpHealthCheck::new();
        balancer_upstreams.set_health_check(hc);
        // TODO: parse from somewhere
        balancer_upstreams.health_check_frequency = Some(time::Duration::from_secs(1));

        let background = background_service("healthcheck", balancer_upstreams);
        // background.task() returns upstreams back.
        let upstreams = background.task();

        let mut balancer = http_proxy_service(&config, Proxy(upstreams));
        // Add a TCP listening endpoint with the given address (e.g., `127.0.0.1:8000`).
        balancer.add_tcp(&proxy_config.listen_addr.as_str());

        balancer
    }
}

pub struct RequestContext {
    pub req_id: uuid::Uuid,
}
