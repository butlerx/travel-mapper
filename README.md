# Travel Mapper

Tool for mapping tripit data

## Setup

```bash
uv sync --frozen
```

## Start server

```bash
uv run fastapi run --host 0.0.0.0 --port 8080 main.py
```

## Dev Server

```bash
robyn server.py ---dev --log-level=DEBUG
uv run fastapi dev --port 8080 main.py
```
