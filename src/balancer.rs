use crate::config;
use async_trait::async_trait;
use bytes::Bytes;
use pingora::{
    http::{self, ResponseHeader},
    server::configuration,
    services::{background::background_service, Service},
    upstreams::peer::HttpPeer,
    Error, Result,
};
use pingora_proxy::{http_proxy_service, ProxyHttp, Session};
use std::sync::Arc;
use std::time;

use uuid;

// TODO: specify selection strategy on construction.
pub struct Proxy(
    Arc<pingora_load_balancing::LoadBalancer<pingora_load_balancing::selection::RoundRobin>>,
);

// Define const to beautify code
const HOST_HEADER_NAME: &str = "target.backend";

// Here lies the main proxy logic.
// The order of method executions is this way:
// 1. request_filter() -> pre-process request. if the request is not intended to be processed further,
// we handle all the logic and terminate the proxy process, writing headers and response.
// 2. upstream_peer() -> selection of the upstream peer
// 3. upstream_request_filter() → request modification before sending to upstream
// 5. response_filter()
// 6. logging()
#[async_trait]
impl ProxyHttp for Proxy {
    type CTX = RequestContext;

    // Here we create a new context for each request. We generate uuid for each request.
    // This should ease potential troubleshooting and logging.
    fn new_ctx(&self) -> Self::CTX {
        RequestContext {
            req_id: uuid::Uuid::new_v4(),
        }
    }
    // Pre-process the request before processing it to upstream_request_filter.
    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        let uri = session.req_header().uri.path().to_string();

        // TODO: normal metrics handler.
        if uri == "/metrics" {
            tracing::info!("Requesting metrics endpoint: {}", uri);

            let mut resp = ResponseHeader::build(http::StatusCode::OK, None)?;
            resp.insert_header("Content-Type", "application/json")?;

            let resp_body: Bytes = format!("{{\"message\":\"ok\"}}").into();

            resp.insert_header("Content-Length", &resp_body.len().to_string())?;
            session.write_response_header(Box::new(resp), true).await?;
            session.write_response_body(Some(resp_body), true).await?;

            // Returning true means that the request has been fully handled and there is no need
            // for further balancing.
            return Ok(true);
        }

        Ok(false)
    }

    // Method responsible for selecting an upstream peer for the given session.
    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // Select upstream from balancer.
        // TODO: customize selection criteria.
        if let Some(upstream) = self.0.select(b"", 256) {
            tracing::info!(
                "Redirecting request {} to {}",
                ctx.req_id.to_string(),
                &upstream.addr
            );

            // Create a peer from the selected upstream.
            // httppeer::new() takes (upstream, use_tls, server_name (SNI))
            // TODO: figure out tls usage.
            let peer = Box::new(HttpPeer::new(upstream, false, HOST_HEADER_NAME.to_string()));

            Ok(peer)
        } else {
            tracing::error!("Failed to select an upstream peer: no healthy upstreams available");
            session.respond_error(500).await?;

            Err(Error::new(pingora::Custom(
                "no healthy upstreams available",
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
        // Provide custom headers.
        upstream_request.insert_header("Host", HOST_HEADER_NAME)?;
        upstream_request.insert_header("X-Request-ID", ctx.req_id.to_string().as_str())?;

        Ok(())
    }

    // Is called after receiving the response from the upstream server.
    async fn response_filter(
        &self,
        _session: &mut Session,
        _upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        Ok(())
    }

    // TODO:
    async fn logging(&self, _session: &mut Session, _e: Option<&Error>, _ctx: &mut Self::CTX) {
        todo!()
    }
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

// Contains metadata about each request.
pub struct RequestContext {
    pub req_id: uuid::Uuid,
}
