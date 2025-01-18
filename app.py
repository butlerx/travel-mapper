"""Main entry point for the application."""

from server import setup_server


def main():
    app = setup_server()
    app.start(host="0.0.0.0", port=8080)


if __name__ == "__main__":
    main()
