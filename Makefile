build:
	docker run --rm -it -v `pwd`:/home/rust/src ekidd/rust-musl-builder cargo build --release
	docker build --no-cache -t risboo6909/when-http .

run:
	docker run --publish 3000:3000 --detach risboo6909/when-http:latest

