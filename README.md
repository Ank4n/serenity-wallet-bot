### Discord bot to collect substrate and ethereum wallets.

##### Discord Configuration
Copy the file `.env.sample`, rename it as `.env` and add the values in the placeholder.
##### Migrations
- Install sqlx cli `cargo install sqlx-cli`
- If you need to add a new migration, use `sqlx migrate add <name>`
- Setup database before running bot `sqlx database setup`
##### Running the bot
- Make sure cargo and rust up are installed and then run the following command from project directory.
`cargo run`
- When adding bot to the server, you need to add `application.command` scope to it. You might also have to give the bot access to show application commands in the channel.

##### Database
- Connect to database `sqlite3 database.sqlite`
- Run sql queries