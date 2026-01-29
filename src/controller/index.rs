use actix_web::{get, HttpResponse, Responder};
use mysql::{Opts, Pool};
use crate::types::conf::Config;
use crate::types::rule::AllRules;

#[get("/")]
pub async fn index_get() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../static/index.html")) // 编译时嵌入
}

