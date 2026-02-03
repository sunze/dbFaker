// use actix_web::{get, HttpResponse, Responder};
// use mysql::{Opts, Pool};
// use crate::types::conf::Config;
// use crate::types::rule::AllRules;
//
//
//
// #[get("/dataset")]
// pub async fn dataset_get() -> impl Responder {
//     HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(include_str!("../../static/dataset.html")) // 编译时嵌入
// }
//
// #[get("/dataset_add.html")]
// pub async fn dataset_add() -> impl Responder {
//     HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(include_str!("../../static/dataset_add.html")) // 编译时嵌入
// }
//
//
// #[get("/dataset_edit.html")]
// pub async fn dataset_edit() -> impl Responder {
//     HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(include_str!("../../static/dataset_edit.html")) // 编译时嵌入
// }
