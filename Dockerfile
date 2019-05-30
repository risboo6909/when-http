FROM scratch

ADD target/x86_64-unknown-linux-musl/release/when-http /

EXPOSE 3000

ENTRYPOINT ["/when-http"]

