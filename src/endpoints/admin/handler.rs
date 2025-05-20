use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

pub async fn dashboard() -> impl IntoResponse {
    let template = AdminTemplate {
        message: String::from("Welcome to your dashboard"),
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "admin.html")]
struct AdminTemplate {
    message: String,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
