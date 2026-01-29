use actix_web::{get, post, web, HttpResponse, Responder};
use mysql::{Opts, Pool};
use mysql::prelude::Queryable;
use crate::middleware::db::{get_conn, get_pool};
use crate::types::conf::Config;
use crate::types::rule::AllRules;

#[get("/api/tables")]
pub async fn index_tables_get() -> impl Responder {
    let pool = get_pool();
    let mut conn = pool.get_conn().expect("Failed to get connection");
    let tables: Vec<String> = conn
        .query("SHOW TABLES")
        .expect("Failed to query tables");

    // 生成 HTML option 标签
    let options_html: String = tables
        .into_iter()
        .map(|t| format!("<option value=\"{0}\">{0}</option>", t))
        .collect();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(options_html)
}



#[derive(serde::Deserialize)]
pub struct TableForm {
    table: String,
}

#[post("/api/table_info")]
pub async fn table_info(form: web::Form<TableForm>) -> impl Responder {
    let msg = format!("你选择的表是：{}", form.table);
    let pool = get_pool();
    let mut conn = pool.get_conn().expect("Failed to get connection");
    // 查询表字段
    let sql = format!(
        "SELECT 
        COLUMN_NAME AS column_name,
        COLUMN_TYPE AS column_type,
        IS_NULLABLE AS is_nullable,
        COLUMN_KEY AS column_key,
        COLUMN_DEFAULT AS column_default,
        EXTRA AS extra,
        COLUMN_COMMENT AS column_comment
    FROM information_schema.COLUMNS 
    WHERE TABLE_SCHEMA = DATABASE()
      AND TABLE_NAME = '{}'
    ORDER BY ORDINAL_POSITION",
        form.table
    );

    #[derive(Debug, mysql::prelude::FromRow)]
    struct ColumnInfo {
        column_name: String,
        column_type: String,
        is_nullable: String,
        column_key: String,
        column_default: Option<String>,
        extra: String,
        column_comment: String,
    }

    let columns: Vec<ColumnInfo> = conn.query(sql).unwrap();

    // 生成 HTML 表格
    let mut html = String::new();
    html.push_str("<table class=\"table-auto border-collapse border border-gray-400 w-full text-sm\">");
    html.push_str("<thead class=\"bg-gray-100\"><tr>");
    html.push_str("<th class=\"border px-2 py-1\">字段名</th>");
    html.push_str("<th class=\"border px-2 py-1\">类型</th>");
    html.push_str("<th class=\"border px-2 py-1\">可空</th>");
    html.push_str("<th class=\"border px-2 py-1\">键</th>");
    html.push_str("<th class=\"border px-2 py-1\">默认值</th>");
    html.push_str("<th class=\"border px-2 py-1\">额外</th>");
    html.push_str("<th class=\"border px-2 py-1\">注释</th>");
    html.push_str("</tr></thead><tbody>");

    for col in columns {
        html.push_str("<tr>");
        html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.column_name));
        html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.column_type));
        html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.is_nullable));
        html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.column_key));
        html.push_str(&format!(
            "<td class=\"border px-2 py-1\">{}</td>",
            col.column_default.clone().unwrap_or("NULL".into())
        ));
        html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.extra));
        html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.column_comment));
        html.push_str("</tr>");
    }

    html.push_str("</tbody></table>");

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}