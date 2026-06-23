use clap::Parser;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::PathBuf;

/// Durchsucht die Mitarbeiterdatenbank nach Vor- oder Nachnamen (Präfix-Suche)
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Suchbegriff (Anfang von Vor- oder Nachname)
    query: String,

    /// Pfad zur SQLite-Datenbank [Standard: employees.db]
    #[arg(short, long, default_value = "employee.db")]
    database: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Verbindung aufbauen & beschleunigen
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{}", args.database.display()))
        .await?;
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA cache_size=-20000")
        .execute(&pool)
        .await?;

    // Präfix-Suche: "baru" -> "baru%"
    let pattern = format!("{}%", args.query.to_lowercase());

    // Nur die nötigsten Felder abfragen
    #[derive(sqlx::FromRow)]
    struct Emp {
        emp_no: i32,
        first_name: String,
        last_name: String,
    }

    let rows: Vec<Emp> = sqlx::query_as(
        "SELECT emp_no, first_name, last_name FROM employees
         WHERE LOWER(first_name) LIKE ?1 OR LOWER(last_name) LIKE ?1
         ORDER BY emp_no
         LIMIT 25",
    )
    .bind(&pattern)
    .fetch_all(&pool)
    .await?;

    let total = rows.len();

    // Kopfzeile
    println!("\n  Mitarbeitersuche nach '{}'", args.query);
    // Gestrichelte Linie
    println!("  {}", "─".repeat(50));

    // Tabellenkopf
    println!(
        "\x1b[1m  {:<6}  {:<15} {:<15}\x1b[0m",
        "ID", "Vorname", "Nachname"
    );
    // Noch eine gestrichelte Linie
    println!("  {}", "─".repeat(50));

    // Ergebniszeilen
    for emp in &rows {
        println!(
            "  {:<6}  {:<15} {:<15}",
            emp.emp_no, emp.first_name, emp.last_name
        );
    }

    // Abschließende gestrichelte Linie
    println!("  {}", "─".repeat(50));
    // Fett gedruckte Trefferzahl (ANSI-Code \x1b[1m ... \x1b[0m)
    println!("\x1b[1m  Total: {} Hits\x1b[0m\n", total);

    Ok(())
}
