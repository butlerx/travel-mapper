FROM python:3.11-bookworm
WORKDIR /workspace
COPY . .
RUN pip install --upgrade pip
RUN pip install --no-cache-dir --upgrade -r requirements.txt
EXPOSE 8080
CMD ["python3", "server.py", "--log-level=INFO"]
