use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{Header, Method, Status},
    Request, Response,
};

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let origin = request.headers().get("origin").last().unwrap_or("*");
        response.set_header(Header::new("Access-Control-Allow-Origin", origin));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(
            Header::new("Access-Control-Allow-Headers", "Access-Control-Allow-Headers, Origin, Accept, X-Requested-With, Content-Type, Access-Control-Request-Method, Access-Control-Request-Headers, auth")
        );
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

pub struct Options;

#[rocket::async_trait]
impl Fairing for Options {
    fn info(&self) -> Info {
        Info {
            name: "Options Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, response: &mut Response<'r>) {
        let method = req.method();
        if method != Method::Options {
            return;
        }

        let req_path = req.uri().path();
        let allowed_methods = req
            .rocket()
            .routes()
            .filter(|r| r.uri.path() == req_path)
            .map(|r| r.method.as_str())
            .collect::<Vec<&str>>();

        if allowed_methods.len() < 1 {
            return;
        }

        let allow = allowed_methods.join(", ");

        response.set_header(Header::new("Allow", allow));
        response.set_status(Status::Ok);
        response.set_sized_body(0, std::io::Cursor::new(""));
    }
}
