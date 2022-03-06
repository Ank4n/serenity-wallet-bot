use std::io::Stderr;



pub struct DbClient {
    database: sqlx::SqlitePool,
}

impl DbClient {
    pub async fn insert_signed(
        &self,
        user_id: String,
        user_tag: String,
        user_name: String,
        ksm_address: String,
        movr_address: String,
        roles: String,
        avatar: String,
    ) -> Option<Stderr> {

        sqlx::query!(
            "INSERT OR REPLACE INTO signed (user_id, user_tag, user_name, ksm_address, movr_address, roles, avatar, create_date) VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))",
             user_id, user_tag, user_name, ksm_address, movr_address, roles, avatar)
        .execute(&self.database)
        .await
        .unwrap();

        None
    }

    pub async fn insert_non_signed(
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
