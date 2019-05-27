FROM ekidd/rust-musl-builder

# We need to add the source code to the image because `rust-musl-builder`
# assumes a UID of 1000, but TravisCI has switched to 2000.
ADD . ./
RUN sudo chown -R rust:rust .

CMD cargo build --release
COPY target/x86_64-unknown-linux-musl/release/when-http /

ENTRYPOINT /when-http

