use std::env;
use std::ops::Add;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use mysql::prelude::Queryable;
use serde::Deserialize;
use num::Complex;
use text_colorizer::Colorize;
use rand::Rng;
use crate::controller::api::{api_dataset_add, api_table_info, api_tables_get};
use crate::controller::index::index_get;
use crate::controller::dataset::{dataset_add, dataset_edit, dataset_get};
pub mod spores;
pub mod plant_structures;
pub mod types;
pub mod controller;
pub mod sink;
pub mod test;
pub mod middleware;
mod database;

#[derive(Debug)]
struct Arguments {
    target: String,
    replacement: String,
    filename: String,
    output: String,
}
#[derive(Deserialize)]
struct GcdParameters {
    n: u64,
    m: u64,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let index = "127.0.0.1:8081";
    println!("http://{}", index);
    HttpServer::new(|| App::new()
        .configure(register_api_services)
        .configure(register_index_services)
        .configure(register_dataset_service))
        .bind(index)?
        .run()
        .await

}


fn register_api_services(cfg: &mut web::ServiceConfig) {
    cfg
        .service(api_tables_get)
        .service(api_table_info)
        .service(api_dataset_add)
        ;
}
fn register_index_services(cfg: &mut web::ServiceConfig) {
    cfg
        .service(index_get);
}

fn register_dataset_service(cfg: &mut web::ServiceConfig) {
    cfg
        .service(dataset_get)
        .service(dataset_add)
        .service(dataset_edit);
}

//
// //#[actix_web::main]
//  fn main() {
//     let config = Config::new().expect("Failed to load config");
//     let rule = AllRules::new().expect("Failed to load rule");
//     //
//
//     println!("Server running at {}:{}", config.server.host, config.server.port);
//     println!("DB url: {}", config.database.url);
//
//     let url = config.database.url;
//     let opts = Opts::from_url(&url).expect("Invalid DB URL");
//     let pool = Pool::new(opts).expect("Failed to create pool");
//     let mut conn = pool.get_conn().expect("Failed to get connection");
//     // 查询并打印表名
//     let tables: Vec<String> = conn
//         .query("SHOW TABLES")
//         .expect("Failed to query tables");
//
//
//     println!("DB tables: {:?}", tables);
//
//
// }

fn parse_args() -> Arguments {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 4 {
        print_usage();
        eprintln!("{} wrong number of arguments: expected 4, got {}.",
                  "Error:".red().bold(), args.len());
        std::process::exit(1);
    }
    Arguments {
        target: args[0].clone(),
        replacement: args[1].clone(),
        filename: args[2].clone(),
        output: args[3].clone()
    }
}

fn print_usage() {
    eprintln!("{} - change occurrences of one string into another",
              "quickreplace".green());
    eprintln!("Usage: quickreplace <target> <replacement> <INPUT> <OUTPUT>");
}

async fn get_index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(
            r#"
<title>GCD Calculator</title>
<form action="/gcd" method="post">
<input type="text" name="n"/>
<input type="text" name="m"/>
<button type="submit">Compute GCD</button>
</form>
"#,
        )
}

fn gcd(mut n: u64, mut m: u64) -> u64 {
    assert!(n != 0 && m != 0);
    while m != 0 {
        if m < n {
            let t = m;
            m = n;
            n = t;
        }
        m = m % n;
    }
    n
}

fn complex_square_add_loop(c: Complex<f64>) {
    let mut z = Complex { re: 0.0, im: 0.0 };
    loop {
        z = z * z + c;
    }
}



#[derive(Deserialize)]
struct FieldString {
    min_length: usize,
    max_length: usize,
    chars: Vec<char>,
}

impl FieldString {
    fn new(min_length: usize, max_length: usize) -> FieldString {
        let alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
        FieldString {
            min_length,
            max_length,
            chars: alphabet,
        }
    }
}

#[derive(Deserialize)]
struct FieldInteger {
    min: i64,
    max: i64,
}

#[derive(Deserialize)]
struct FieldEmail {

}


fn fake_string(field: FieldString) -> String {
    let mut rng = rand::rng();
    let length = rng.random_range(field.min_length..=field.max_length);
    let mut result = String::with_capacity(length);
    for _ in 0..length {
        let idx = rng.random_range(0..field.chars.len());
        result.push(field.chars[idx]);
    }
    result
}

fn fake_int(field: FieldInteger) -> i64 {
    let mut rng = rand::rng();
    rng.random_range(field.min..=field.max)
}

fn fake_email(field: FieldEmail) -> String {
    let domains = vec![
        "gmail.com",
        "qq.com",
        "163.com",
        "outlook.com",
        "yahoo.com",
        "hotmail.com",
        "yandex.com",
        "mail.ru",
        "aol.com",
        "icloud.com",
        "live.com",
        "zoho.com",
        "protonmail.com",
        "fastmail.com",
        "gmx.com",
        "mail.com",
     ];
    let mut rng = rand::rng();
    let domain = domains[rng.random_range(0..domains.len())];
    let mut result = "test".to_string();
    result.push_str("@");
    result.push_str(domain);
    result
}

#[test]
fn test_fake_email() {
    let field = FieldEmail{};
    let s = fake_email(field);
    println!("随机字符串：{}", s)
}

#[test]
fn test_fake_string_length() {
    let field = FieldString::new(6, 10);
    let s = fake_string(field);
    assert!(s.len() >= 6 && s.len() <= 10, "长度不在范围内");
    println!("随机字符串：{}", s)

}

#[test]
fn test_fake_int() {
    let field = FieldInteger {min: 1, max: 100};
    let n = fake_int(field);
    assert!(n >= 1 && n <= 100, "不在范围内");
    println!("随机整数：{}", n)
}
