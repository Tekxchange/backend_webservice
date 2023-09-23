use std::net::UdpSocket;

use cadence::prelude::*;
use cadence::StatsdClient;
use cadence::UdpMetricSink;
use chrono::{DateTime, Utc};
use rocket::{
    fairing::{Fairing, Kind},
    Data, Request, Response,
};

#[derive(Clone, Copy, Debug)]
struct RequestTimer(Option<DateTime<Utc>>);

pub struct Statsd {
    client: StatsdClient,
}

impl Default for Statsd {
    fn default() -> Self {
        let host = std::env::var("STATSD_HOST").expect("Unable to get statsd host from env var");
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Unable to bind a statsd socket");
        let sink = UdpMetricSink::from(host, socket).expect("Unable to create a metrics sink");
        let client = StatsdClient::from_sink("tekxchange", sink);

        Self { client }
    }
}

#[rocket::async_trait]
impl Fairing for Statsd {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            kind: Kind::Request | Kind::Response,
            name: "Statsd metrics",
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        let start_time = Utc::now();
        req.local_cache(|| RequestTimer(Some(start_time)));
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let method = request.method().as_str();
        let path = request.uri().path().as_str();
        let status = response.status().code;

        let found_count = request
            .rocket()
            .routes()
            .filter(|r| {
                let p = r.uri.path();
                return r.method == request.method() && p == path;
            })
            .count();

        if found_count < 1 {
            return;
        }

        let path = path.replace("/", ".");

        let stat = format!("request.{method}{path}");

        let end_time = Utc::now().time();
        let start_time = *request.local_cache(|| RequestTimer(None));
        if let Some(start_time) = start_time.0 {
            let diff = (end_time - start_time.time()).num_milliseconds() as u64;
            let _ = self.client.time(&stat, diff);
        }
        let _ = self.client
            .incr(&format!("request.{method}{path}.{status}"));
    }
}
