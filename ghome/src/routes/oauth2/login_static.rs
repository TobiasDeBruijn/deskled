use actix_web::HttpResponse;
use tracing::instrument;

const STATIC_PAGE: &[u8] = include_bytes!("login_static.html");

#[instrument]
pub async fn login_static() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(STATIC_PAGE)
}
