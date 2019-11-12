FROM rust:1.31

WORKDIR /usr/src/ise
COPY . .

RUN cargo install cargo-web
# 
# RUN cargo web start

CMD ["cargo", "web", "start"]
