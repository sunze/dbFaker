// use actix_web::{get, post, web, HttpResponse, Responder};
// use mysql::{Opts, Pool};
// use mysql::prelude::Queryable;
// use crate::database::database::DatasetDb;
// use crate::middleware::db::{get_conn, get_pool};
// use crate::types::conf::Config;
// use crate::types::rule::AllRules;
//
// #[get("/api/tables")]
// pub async fn api_tables_get() -> impl Responder {
//     let pool = get_pool();
//     let mut conn = pool.get_conn().expect("Failed to get connection");
//     let tables: Vec<String> = conn
//         .query("SHOW TABLES")
//         .expect("Failed to query tables");
//
//     // 生成 HTML option 标签
//     let options_html: String = tables
//         .into_iter()
//         .map(|t| format!("<option value=\"{0}\">{0}</option>", t))
//         .collect();
//     HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(options_html)
// }
//
//
// #[get("/api/datasets")]
// pub async fn api_dataset_get() -> impl Responder {
//     let db = match DatasetDb::init() {
//         Ok(db) => db,
//         Err(e) => {
//             log::error!("init db failed: {:?}", e);
//             return HttpResponse::InternalServerError()
//                 .body("数据库初始化失败");
//         }
//     };
//
//     let datasets = match db.get_all_datasets() {
//         Ok(list) => list,
//         Err(e) => {
//             log::error!("query datasets failed: {:?}", e);
//             return HttpResponse::InternalServerError()
//                 .body("查询数据集失败");
//         }
//     };
//
//     let mut html = String::new();
//
//     if datasets.is_empty() {
//         html.push_str(r#"
//         <tr>
//             <td colspan="4" class="px-6 py-10 text-center text-[#6a988f]">
//                 暂无数据集，点击「新建数据集」添加
//             </td>
//         </tr>
//         "#);
//     } else {
//         for d in datasets {
//             html.push_str(&format!(r#"
//             <tr class="border-b border-[#d1e9e4] hover:bg-[#f0f7f5] transition">
//                 <td class="px-6 py-4">{}</td>
//                 <td class="px-6 py-4">{}</td>
//                 <td class="px-6 py-4">{}</td>
//                 <td class="px-6 py-4 flex gap-3">
//                     <a href="dataset_edit.html?id={}"
//                        class="text-[#5fb878] hover:text-[#52a86d] transition">
//                         编辑
//                     </a>
//                     <button
//                         class="text-[#f56c6c] hover:text-[#e64949]"
//                         hx-delete="/api/dataset/{}"
//                         hx-confirm="确定删除该数据集吗？"
//                         hx-target="closest tr"
//                         hx-swap="outerHTML">
//                         删除
//                     </button>
//                 </td>
//             </tr>
//             "#,
//                                    d.name,
//                                    d.set_type,
//                                    d.create_time,
//                                    d.id,
//                                    d.id
//             ));
//         }
//     }
//     HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(html)
// }
//
//
//
// #[derive(serde::Deserialize)]
// pub struct TableForm {
//     table: String,
// }
//
// #[derive(serde::Deserialize)]
// pub struct DatasetAddForm {
//     dataset_name: String,
//     dataset_file: String,
// }
//
//
// #[post("/api/dataset_add")]
// pub async fn api_dataset_add(form: web::Form<DatasetAddForm>) -> impl Responder {
//     let db = match DatasetDb::init() {
//         Ok(db) => db,
//         Err(e) => {
//             log::error!("init db failed: {:?}", e);
//             return HttpResponse::InternalServerError()
//                 .body("数据库初始化失败");
//         }
//     };
//
//     let dataset_id = match db.add_dataset(&form.dataset_name, "string") {
//         Ok(id) => id,
//         Err(e) => {
//             log::error!("add_dataset failed: {:?}", e);
//             return HttpResponse::InternalServerError()
//                 .body("新增数据集失败");
//         }
//     };
//
//     HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(format!("新增数据集ID: {}", dataset_id))
// }
//
//
// #[post("/api/table_info")]
// pub async fn api_table_info(form: web::Form<TableForm>) -> impl Responder {
//     let msg = format!("你选择的表是：{}", form.table);
//     let pool = get_pool();
//     let mut conn = pool.get_conn().expect("Failed to get connection");
//     // 查询表字段
//     let sql = format!(
//         "SELECT
//         COLUMN_NAME AS column_name,
//         COLUMN_TYPE AS column_type,
//         IS_NULLABLE AS is_nullable,
//         COLUMN_KEY AS column_key,
//         COLUMN_DEFAULT AS column_default,
//         EXTRA AS extra,
//         COLUMN_COMMENT AS column_comment
//     FROM information_schema.COLUMNS
//     WHERE TABLE_SCHEMA = DATABASE()
//       AND TABLE_NAME = '{}'
//     ORDER BY ORDINAL_POSITION",
//         form.table
//     );
//
//     #[derive(Debug, mysql::prelude::FromRow)]
//     struct ColumnInfo {
//         column_name: String,
//         column_type: String,
//         is_nullable: String,
//         column_key: String,
//         column_default: Option<String>,
//         extra: String,
//         column_comment: String,
//     }
//
//     let columns: Vec<ColumnInfo> = conn.query(sql).unwrap();
//
//     // 生成 HTML 表格
//     let mut html = String::new();
//     html.push_str("<table class=\"table-auto border-collapse border border-gray-400 w-full text-sm\">");
//     html.push_str("<thead class=\"bg-gray-100\"><tr>");
//     html.push_str("<th class=\"border px-2 py-1\">字段名</th>");
//     html.push_str("<th class=\"border px-2 py-1\">类型</th>");
//     html.push_str("<th class=\"border px-2 py-1\">可空</th>");
//     html.push_str("<th class=\"border px-2 py-1\">键</th>");
//     html.push_str("<th class=\"border px-2 py-1\">默认值</th>");
//     html.push_str("<th class=\"border px-2 py-1\">额外</th>");
//     html.push_str("<th class=\"border px-2 py-1\">注释</th>");
//     html.push_str("</tr></thead><tbody>");
//
//     for col in columns {
//         html.push_str("<tr>");
//         html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.column_name));
//         html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.column_type));
//         html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.is_nullable));
//         html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.column_key));
//         html.push_str(&format!(
//             "<td class=\"border px-2 py-1\">{}</td>",
//             col.column_default.clone().unwrap_or("NULL".into())
//         ));
//         html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.extra));
//         html.push_str(&format!("<td class=\"border px-2 py-1\">{}</td>", col.column_comment));
//         html.push_str("</tr>");
//     }
//
//     html.push_str("</tbody></table>");
//
//     HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(html)
// }