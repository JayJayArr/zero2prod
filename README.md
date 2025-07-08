### zero2prod

A full implementation of the functionality from the Book "Zero to production in Rust" by Luca Palmieri using axum instead of actix-web.
Essentially a small blog application that sends out emails for each issue of the blog and offers a small web interface.

To run the project an instance of Postgres and Redis is needed.
To start both in a docker environment run:

```Shell
./scripts/init_db.sh
./scripts/init_redis.sh
```

The application can then be started using:

```Shell
cargo run

```

The project uses two components side by side:

- The API itself
- A background worker sending out the mails
