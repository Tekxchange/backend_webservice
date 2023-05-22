use redis::{AsyncCommands, Client as RedisClient};
use rocket::{
    http::{Header, Status},
    outcome::{try_outcome, Outcome},
    request::FromRequest,
    response::Responder,
    response::Response,
    State,
};

#[derive(Debug)]
pub struct TooManyRequests {
    amount: i16,
    time_left: i32,
}

impl<'r> Responder<'r, 'static> for TooManyRequests {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        Response::build()
            .status(Status::TooManyRequests)
            .header(Header::new("x-rate-limit", self.amount.to_string()))
            .header(Header::new(
                "x-rate-limit-reset",
                self.time_left.to_string(),
            ))
            .ok()
    }
}

pub struct AuthRateLimit {
    pub request_amount: i16,
}

#[async_trait]
impl<'r> FromRequest<'r> for AuthRateLimit {
    type Error = ();

    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let redis: &RedisClient =
            try_outcome!(request.guard::<&State<RedisClient>>().await).inner();
        let mut conn = match redis.get_async_connection().await {
            Ok(con) => con,
            Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
        };
        let path = request.uri().path().as_str();

        let ip_str = match request.client_ip() {
            Some(ip) => ip,
            None => return Outcome::Failure((Status::TooManyRequests, ())),
        }
        .to_string();

        let key = format!("{ip_str}{path}");

        let amount = match conn.get::<&str, Option<i16>>(&key).await {
            Ok(amt) => amt,
            Err(_) => return Outcome::Failure((Status::InternalServerError, ())),
        }
        .unwrap_or_else(|| 0)
            + 1;

        match conn.set_ex::<&str, i16, ()>(&key, amount, 300).await {
            Ok(_) => Outcome::Success(Self {
                request_amount: amount,
            }),
            Err(_) => Outcome::Failure((Status::InternalServerError, ())),
        }
    }
}
