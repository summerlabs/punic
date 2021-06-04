build:
	cargo build
	cp -f target/debug/punic /usr/local/bin/punic

release:
	cargo build --release
	cd target/release \
		&& tar -cvf punic.tar.gz punic \
		&& shasum -a 256 punic.tar.gz
