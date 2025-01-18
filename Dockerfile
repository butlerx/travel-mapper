FROM python:3.11-bookworm

WORKDIR /workspace

COPY . .

RUN python pip install --no-cache-dir --upgrade -r requirements.txt
RUN python -m rustimport build --release server/database

EXPOSE 8080

CMD ["python3", "app.py", "--log-level=DEBUG"]
