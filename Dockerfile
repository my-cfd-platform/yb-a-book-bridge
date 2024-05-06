FROM ubuntu:22.04
COPY ./target/release/yb-a-book-bridge ./target/release/yb-a-book-bridge

ENTRYPOINT ["./target/release/yb-a-book-bridge"]