build:
	cargo build
	cp -f target/debug/punic /usr/local/bin/punic

release: build
	cd target/release && tar -cvf punic.tar.gz punic
