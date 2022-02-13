use std::io::Error as Stderr;

pub struct DbClient {
    database: sqlx::SqlitePool,
}

impl DbClient {
    pub async fn insert(
        &self,
        user_id: String,
        user_name: String,
        address_type: String,
        address: String,
        roles: String,
        avatar: String,
    ) -> Option<Stderr> {

        sqlx::query!(
            "INSERT OR REPLACE INTO users (user_id, user_name, address_type, address, roles, avatar, create_date, update_date) VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
             user_id, user_name, address_type, address, roles, avatar)
        .execute(&self.database)
        .await
        .unwrap();

        None
    }
}

pub async fn init(filename: String) -> DbClient {
    // Initiate a connection to the database file, creating the file if required.
    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(filename)
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");

    // Run migrations, which updates the database's schema to the latest version.
    sqlx::migrate!("./migrations")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");

    DbClient { database }
}
