use axum::extract::Path; // Json statt Html
use axum::extract::State;
use axum::{
    Json, Router, extract::Form, http::StatusCode, response::Html, routing::get, routing::post,
};
use serde::Deserialize;
use serde::Serialize;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

// Logging
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize)]
struct HealthStatus {
    status: String,
    database: String,
}

async fn health(State(pool): State<SqlitePool>) -> (StatusCode, Json<HealthStatus>) {
    // Einfacher Datenbank‑Ping: versuche eine triviale Abfrage
    let db_ok = sqlx::query("SELECT 1").fetch_one(&pool).await.is_ok();

    let status = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let body = HealthStatus {
        status: if db_ok { "ok".into() } else { "error".into() },
        database: if db_ok {
            "connected".into()
        } else {
            "disconnected".into()
        },
    };

    (status, Json(body))
}

#[derive(Deserialize)]
struct SearchForm {
    q: String,
}

#[derive(Debug, sqlx::FromRow)]
struct SimpleEmployee {
    emp_no: i32,
    first_name: String,
    last_name: String,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
struct EmployeeDetail {
    emp_no: i32,
    first_name: String,
    last_name: String,
    birth_date: String, // Datum als String für JSON/HTML
    hire_date: String,
    dept_name: String,
    title: String,
    salary: i32,
    from_date: String,
    to_date: String,
}

async fn search(State(pool): State<SqlitePool>, Form(form): Form<SearchForm>) -> Html<String> {
    let query = form.q.trim().to_lowercase();
    if query.is_empty() {
        return Html(String::new());
    }

    tracing::info!("Suche nach '{}'", query);

    let pattern = format!("%{}%", query);
    let rows = sqlx::query_as::<_, SimpleEmployee>(
        "SELECT emp_no, first_name, last_name FROM employees
         WHERE LOWER(first_name) LIKE ?1 OR LOWER(last_name) LIKE ?1
         LIMIT 50",
    )
    .bind(pattern)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let html = if rows.is_empty() {
        "<tr><td colspan='2'>Keine Treffer</td></tr>".to_string()
    } else {
        rows.iter()
            .map(|e| {
                format!(
                    "<tr><td>{}, {}</td><td>{}</td></tr>",
                    e.last_name, e.first_name, e.emp_no
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    Html(html)
}

async fn employee_detail(State(pool): State<SqlitePool>, Path(emp_no): Path<i32>) -> Html<String> {
    let detail = sqlx::query_as::<_, EmployeeDetail>(
        "SELECT e.emp_no, e.first_name, e.last_name,
                        e.birth_date, e.hire_date,
                        d.dept_name,
                        ct.title,
                        cs.salary, cs.from_date, cs.to_date
                 FROM employees e
                 JOIN current_dept_emp cde ON e.emp_no = cde.emp_no
                 JOIN departments d ON cde.dept_no = d.dept_no
                 LEFT JOIN current_salary cs ON e.emp_no = cs.emp_no
                 LEFT JOIN current_title ct ON e.emp_no = ct.emp_no
                 WHERE e.emp_no = ?1",
    )
    .bind(emp_no)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    match detail {
        Some(emp) => {
            let html = format!(
                "<div class='employee-detail'>
                    <h3>{} {}, {}</h3>
                    <p>Geboren: {}, Einstellung: {}</p>
                    <p>Abteilung: {}</p>
                    <p>Titel: {} (seit {})</p>
                    <p>Gehalt: ${} (von {} bis {})</p>
                </div>",
                emp.first_name,
                emp.last_name,
                emp.emp_no,
                emp.birth_date,
                emp.hire_date,
                emp.dept_name,
                emp.title,
                emp.from_date,
                emp.salary,
                emp.from_date,
                emp.to_date
            );
            Html(html)
        }
        None => Html("<p>Mitarbeiter nicht gefunden.</p>".to_string()),
    }
}

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:employee.db")
        .await
        .unwrap();
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("PRAGMA cache_size=-20000")
        .execute(&pool)
        .await
        .unwrap(); // 20 MB Cache
    sqlx::query("PRAGMA synchronous=NORMAL")
        .execute(&pool)
        .await
        .unwrap();

    let app = Router::new()
        .route("/health", get(health))
        .route("/search", post(search))
        .route("/employee/:id/detail", get(employee_detail))
        .layer(TraceLayer::new_for_http())
        .with_state(pool); // <-- Pool direkt, kein extra Arc

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("Axum server läuft auf 127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
