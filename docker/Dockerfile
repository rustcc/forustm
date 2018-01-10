#FROM library/rust
FROM ubuntu

WORKDIR /home/forustm

RUN apt-get update && apt-get install -y libpq-dev libssl1.0.0

#COPY .. .
#RUN cargo install

ADD .env .

CMD ["sh"]
