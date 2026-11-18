//! A rate limit by client address with one allowance for IPv4 and another for IPv6. An IPv6
//! client counts by its /64, which a household or an office shares, so it gets more room than
//! one IPv4 address; an IPv4 address written as IPv6 counts as IPv4.

use std::sync::Arc;
use std::task::{Context, Poll, ready};
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, Response};
use governor::middleware::NoOpMiddleware;
use tower::{Layer, Service};
use tower_governor::governor::{Governor, GovernorConfig, GovernorConfigBuilder};
use tower_governor::{GovernorError, GovernorLayer};

use crate::inbound::http::limiter_purge::{LIMITER_PURGE_EVERY, purging};
use crate::inbound::http::middleware::{CfConnectingIpKeyExtractor, client_key};

/// One family's allowance: `burst` requests at once, then one more each `period`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Allowance {
    /// How long one request takes to come back.
    pub period: Duration,
    /// How many requests go at once.
    pub burst: u32,
}

impl Allowance {
    /// `burst` at once, then one each `period`.
    #[must_use]
    pub const fn new(period: Duration, burst: u32) -> Self {
        Self { period, burst }
    }
}

/// A governor config keyed by client address.
type AddressConfig = GovernorConfig<CfConnectingIpKeyExtractor, NoOpMiddleware>;

/// A governor layer keyed by client address.
type AddressLayer = GovernorLayer<CfConnectingIpKeyExtractor, NoOpMiddleware, Body>;

/// The layer for an address-keyed limit with an `ipv4` and an `ipv6` allowance, each answering a
/// refusal through `on_refuse`. Cloning it shares its counts, as a shared config does.
#[derive(Clone)]
pub struct FamilyLimitLayer {
    ipv4: AddressLayer,
    ipv6: AddressLayer,
}

impl std::fmt::Debug for FamilyLimitLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FamilyLimitLayer").finish_non_exhaustive()
    }
}

impl FamilyLimitLayer {
    /// The limit with `ipv4`'s allowance for an IPv4 client and `ipv6`'s for an IPv6 /64.
    ///
    /// # Panics
    ///
    /// When an allowance has a zero period or burst, which governor can't build; every caller
    /// passes non-zero constants.
    #[must_use]
    pub fn new(
        ipv4: Allowance,
        ipv6: Allowance,
        on_refuse: fn(GovernorError) -> Response<Body>,
    ) -> Self {
        Self {
            ipv4: GovernorLayer::new(config(ipv4)).error_handler(on_refuse),
            ipv6: GovernorLayer::new(config(ipv6)).error_handler(on_refuse),
        }
    }
}

/// `allowance` as a governor config keyed by client address, built through [`purging`] so the
/// clients whose budget has refilled are let go.
#[expect(
    clippy::expect_used,
    reason = "every allowance has a non-zero period and burst, so finish never returns None"
)]
fn config(allowance: Allowance) -> Arc<AddressConfig> {
    purging(
        GovernorConfigBuilder::default()
            .period(allowance.period)
            .burst_size(allowance.burst)
            .key_extractor(CfConnectingIpKeyExtractor)
            .finish()
            .expect("rate limit config: burst_size and period must be non-zero"),
        LIMITER_PURGE_EVERY,
    )
}

impl<S: Clone> Layer<S> for FamilyLimitLayer {
    type Service = FamilyLimit<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FamilyLimit {
            ipv4: self.ipv4.layer(inner.clone()),
            ipv6: self.ipv6.layer(inner),
        }
    }
}

/// The service [`FamilyLimitLayer`] builds: the inner service behind both limits, each request
/// counted against its client's family. It is ready once both limits' inner services are, then
/// calls one; that holds no slot while the inner service is an always-ready route, but an inner
/// service with real backpressure would keep the unused side's slot.
#[derive(Clone)]
pub struct FamilyLimit<S> {
    ipv4: Governor<CfConnectingIpKeyExtractor, NoOpMiddleware, S, Body>,
    ipv6: Governor<CfConnectingIpKeyExtractor, NoOpMiddleware, S, Body>,
}

impl<S> std::fmt::Debug for FamilyLimit<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FamilyLimit").finish_non_exhaustive()
    }
}

impl<S, ReqBody> Service<Request<ReqBody>> for FamilyLimit<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>>,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = <Governor<CfConnectingIpKeyExtractor, NoOpMiddleware, S, Body> as Service<
        Request<ReqBody>,
    >>::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.ipv4.poll_ready(cx))?;
        self.ipv6.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        // A request with no address goes to the IPv4 limit, whose key extractor refuses it.
        if client_key(req.headers(), req.extensions()).is_some_and(|ip| ip.is_ipv6()) {
            self.ipv6.call(req)
        } else {
            self.ipv4.call(req)
        }
    }
}
