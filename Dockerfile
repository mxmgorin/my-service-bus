FROM rust
COPY . . 
ENTRYPOINT ["./target/release/my-service-bus"]