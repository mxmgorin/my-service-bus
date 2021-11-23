FROM rust:slim
COPY ./target/release/my-service-bus ./target/release/my-service-bus 
ENTRYPOINT ["./target/release/my-service-bus"]