# Deployment Guide

Travel Mapper is a single Rust binary (`travel_mapper`) with two subcommands:
`serve` starts the HTTP server and `worker` runs background sync. Both share the
same database and configuration. For environment variables and optional
integrations, see the
[Setup and Configuration Guide](setup-and-configuration.md).

## Building for production

```bash
cargo build --release --locked
```

This produces `target/release/travel_mapper`. LTO and symbol stripping are
configured in `Cargo.toml` so no extra flags are needed. The binary is
statically linked against Rust's standard library and only depends on the
system's libc.

## Required environment

Three environment variables are required:

- `TRIPIT_CONSUMER_KEY` — TripIt OAuth consumer key
- `TRIPIT_CONSUMER_SECRET` — TripIt OAuth consumer secret
- `ENCRYPTION_KEY` — 64 hex characters (32 bytes) for AES-256-GCM

`DATABASE_URL` defaults to `sqlite:travel.db` in the working directory.
Migrations run automatically when the connection pool initialises — no manual
migration step is needed.

See the [Setup and Configuration Guide](setup-and-configuration.md) for the
full environment variables reference.

## Running manually

```bash
./travel_mapper serve
./travel_mapper worker
```

The server binds to `0.0.0.0:3000` by default (override with `--port` or
`PORT`). Both subcommands support graceful shutdown via SIGINT.

## systemd units

### Server

Create `/etc/systemd/system/travel-mapper-server.service`:

```ini
[Unit]
Description=Travel Mapper web server
After=network.target

[Service]
Type=exec
ExecStart=/usr/local/bin/travel_mapper serve
EnvironmentFile=/etc/travel-mapper/env
WorkingDirectory=/var/lib/travel-mapper
DynamicUser=yes
StateDirectory=travel-mapper
KillSignal=SIGINT
Restart=on-failure
RestartSec=5

ProtectSystem=strict
ProtectHome=yes
NoNewPrivileges=yes
PrivateTmp=yes
ReadWritePaths=/var/lib/travel-mapper

[Install]
WantedBy=multi-user.target
```

### Worker

Create `/etc/systemd/system/travel-mapper-worker.service`:

```ini
[Unit]
Description=Travel Mapper sync worker
After=network.target
PartOf=travel-mapper-server.service

[Service]
Type=exec
ExecStart=/usr/local/bin/travel_mapper worker
EnvironmentFile=/etc/travel-mapper/env
WorkingDirectory=/var/lib/travel-mapper
DynamicUser=yes
StateDirectory=travel-mapper
KillSignal=SIGINT
Restart=on-failure
RestartSec=5

ProtectSystem=strict
ProtectHome=yes
NoNewPrivileges=yes
PrivateTmp=yes
ReadWritePaths=/var/lib/travel-mapper

[Install]
WantedBy=multi-user.target
```

The `PartOf` directive ensures the worker stops and restarts together with the
server.

## Installation steps

1. Copy the binary:

   ```bash
   sudo install -m 755 target/release/travel_mapper /usr/local/bin/travel_mapper
   ```

2. Create the environment file:

   ```bash
   sudo mkdir -p /etc/travel-mapper
   sudo tee /etc/travel-mapper/env > /dev/null <<'EOF'
   TRIPIT_CONSUMER_KEY=your_consumer_key
   TRIPIT_CONSUMER_SECRET=your_consumer_secret
   ENCRYPTION_KEY=your_64_hex_char_key
   DATABASE_URL=sqlite:/var/lib/travel-mapper/travel.db
   EOF
   sudo chmod 600 /etc/travel-mapper/env
   ```

3. Copy the unit files to `/etc/systemd/system/`.

4. Reload systemd and start:

   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable --now travel-mapper-server travel-mapper-worker
   ```

## Reverse proxy

### nginx

```nginx
server {
    listen 80;
    server_name travel.example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

### Caddy

```
travel.example.com {
    reverse_proxy 127.0.0.1:3000
}
```

## Viewing logs

```bash
journalctl -u travel-mapper-server -f
journalctl -u travel-mapper-worker -f
```

Release builds emit JSON-formatted logs (one object per line). Configure
verbosity with the `RUST_LOG` environment variable in the env file — see the
[Logging and Telemetry](setup-and-configuration.md#logging-and-telemetry)
section for syntax.

## Updating

1. Stop both services:

   ```bash
   sudo systemctl stop travel-mapper-server
   ```

   This also stops the worker via `PartOf`.

2. Replace the binary:

   ```bash
   sudo install -m 755 target/release/travel_mapper /usr/local/bin/travel_mapper
   ```

3. Start the services:

   ```bash
   sudo systemctl start travel-mapper-server
   ```

   Database migrations run automatically on startup.
