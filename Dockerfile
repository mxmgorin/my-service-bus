FROM rust:slim
COPY ./target/release/my-service-bus ./target/release/my-service-bus 
COPY ./wwwroot ./wwwroot 
ENTRYPOINT ["./target/release/my-service-bus"]