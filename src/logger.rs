use reqwest::Url;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn setup_loki() {
    let loki_host = std::env::var("LOKI_SERVER").unwrap();

    let (layer, task) = tracing_loki::builder()
        .label("service", "tekxchange_backend_webservice")
        .unwrap()
        .build_url(Url::parse(&loki_host).unwrap())
        .unwrap();

    tracing_subscriber::registry().with(layer).init();

    rocket::tokio::spawn(task);
}
