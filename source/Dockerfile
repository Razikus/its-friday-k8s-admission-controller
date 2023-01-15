FROM python:3.11
COPY requirements.txt /requirements.txt
RUN pip3 install -r /requirements.txt
RUN mkdir /certs
COPY app /app/app

WORKDIR /app

ENTRYPOINT ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--ssl-keyfile", "/app/certs/tls.key", "--ssl-certfile", "/app/certs/tls.crt", "--port", "443"]