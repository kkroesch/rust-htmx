Hier noch einmal kompakt alle Änderungen, die wir an deiner SQLite-Datenbank vorgenommen haben, damit du sie reproduzieren oder dokumentieren kannst. Die Befehle sind **additiv** – sie verändern keine bestehenden Daten und können gefahrlos auf dem Original ausgeführt werden.

---

### 1. Laufzeit-PRAGMAs (im Rust-Code nach dem Pool-Aufbau)

```rust
sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await.unwrap();
sqlx::query("PRAGMA cache_size=-20000").execute(&pool).await.unwrap();
sqlx::query("PRAGMA synchronous=NORMAL").execute(&pool).await.unwrap();
```

**Wirkung:** Stellt auf Write‑Ahead‑Logging um, erhöht den Cache auf 20 MB und reduziert Synchronisations‑Overhead. Diese Einstellungen gelten nur für die aktuelle Verbindung und werden nicht in der Datenbankdatei gespeichert.

---

### 2. Indizes (einmalig in der DB anzulegen)

```sql
CREATE INDEX IF NOT EXISTS idx_dept_emp_emp_from
    ON dept_emp(emp_no, from_date DESC);
CREATE INDEX IF NOT EXISTS idx_salaries_emp_from
    ON salaries(emp_no, from_date DESC);
CREATE INDEX IF NOT EXISTS idx_titles_emp_from
    ON titles(emp_no, from_date DESC);
```

**Zweck:** Beschleunigen die Suche nach dem maximalen `from_date` pro Mitarbeiter, die in den Views `current_salary` und `current_title` sowie in der Detailabfrage verwendet wird.

---

### 3. Neue Views für aktuelle Daten

```sql
CREATE VIEW IF NOT EXISTS current_salary AS
SELECT s.emp_no, s.salary, s.from_date, s.to_date
FROM salaries s
INNER JOIN (
    SELECT emp_no, MAX(from_date) AS max_from
    FROM salaries
    WHERE to_date = '9999-01-01'
    GROUP BY emp_no
) latest ON s.emp_no = latest.emp_no AND s.from_date = latest.max_from
WHERE s.to_date = '9999-01-01';

CREATE VIEW IF NOT EXISTS current_title AS
SELECT t.emp_no, t.title, t.from_date, t.to_date
FROM titles t
INNER JOIN (
    SELECT emp_no, MAX(from_date) AS max_from
    FROM titles
    WHERE to_date = '9999-01-01'
    GROUP BY emp_no
) latest ON t.emp_no = latest.emp_no AND t.from_date = latest.max_from
WHERE t.to_date = '9999-01-01';
```

**Zweck:** Materialisieren den aktuellen Titel und das aktuelle Gehalt jedes Mitarbeiters (definiert durch das maximale `from_date` mit `to_date = '9999-01-01'`). Diese Views ersetzen die langsamen Unterabfragen, die ursprünglich in der Detailabfrage steckten.

---

### 4. (Optional) Volltextindex für blitzschnelle Namenssuche

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS employees_fts
USING fts5(first_name, last_name, content=employees, content_rowid=emp_no);
```

**Wirkung:** Ermöglicht `MATCH`-Abfragen, die auch bei `LIKE '%...%'`-Mustern in großen Tabellen nur wenige Millisekunden brauchen.  
**Hinweis:** Da die Tabelle ein virtueller Index ist, muss sie bei Änderungen an `employees` manuell aktualisiert werden (für unsere Demo irrelevant).

---

### 5. Zusammenfassung

| Maßnahme               | Befehl(e)                      | Bleibt persistent? | Angewandt in…        |
|------------------------|--------------------------------|--------------------|----------------------|
| PRAGMAs                | `PRAGMA journal_mode=WAL;` etc.| Nein (pro Session) | Rust-Code            |
| Indizes                | `CREATE INDEX IF NOT EXISTS …` | Ja                 | `sqlite3 employees.db` |
| Views                  | `CREATE VIEW IF NOT EXISTS …`  | Ja                 | `sqlite3 employees.db` |
| FTS5 (optional)        | `CREATE VIRTUAL TABLE …`       | Ja                 | `sqlite3 employees.db` |

Wenn du das Original zurücksetzen willst, genügt:

```sql
DROP INDEX idx_dept_emp_emp_from;
DROP INDEX idx_salaries_emp_from;
DROP INDEX idx_titles_emp_from;
DROP VIEW current_salary;
DROP VIEW current_title;
DROP TABLE IF EXISTS employees_fts;
```

Jetzt hast du alle Änderungen sauber dokumentiert – die Suche bleibt schnell und die Basis passt wieder zum ursprünglichen Schema.
