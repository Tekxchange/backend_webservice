use reqwest::Url;
use rocket::fairing::{Fairing, Kind};
use rocket::{Data, Request, Response};
use tracing::Span;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter, Layer};

pub fn setup_loki() {
    let loki_host = std::env::var("LOKI_SERVER").unwrap();

    let filter =
        filter::dynamic_filter_fn(|f, _| f.target().starts_with("rust_tekxchange_backend"));

    let (layer, task) = tracing_loki::builder()
        .label("service", "rust_tekxchange_backend")
        .unwrap()
        .build_url(Url::parse(&loki_host).unwrap())
        .unwrap();

    tracing_subscriber::registry()
        .with(layer.with_filter(filter))
        .init();

    rocket::tokio::spawn(task);
}

pub struct Loki;

#[rocket::async_trait]
impl Fairing for Loki {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            kind: Kind::Request | Kind::Response,
            name: "Loki logger",
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        let uri = req.uri().to_string();
        let span = tracing::error_span!("new_request", uri);
        req.local_cache(|| Some(span));
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        
        let status = response.status();
    }
}
