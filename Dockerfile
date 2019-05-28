FROM scratch 

ADD target/x86_64-unknown-linux-musl/release/when-http /

ENTRYPOINT ["/when-http"]

