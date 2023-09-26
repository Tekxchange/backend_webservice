use reqwest::Url;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter, Layer};

pub fn setup_loki() {
    let loki_host = std::env::var("LOKI_SERVER").unwrap();

    let filter = filter::dynamic_filter_fn(|f, _| f.target() == "rust_tekxchange_backend");

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
