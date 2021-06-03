build:
	cargo build
	cp -f target/debug/punic /usr/local/bin/punic

release: build
	tar -cvf target/release/punic.tar.gz target/release/punic
