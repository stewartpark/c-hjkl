all: build

build:
	cargo build --release

install: build
	sudo install target/release/c-hjkl /usr/local/bin
	sudo install systemd/c-hjkl.service /etc/systemd/system
	sudo systemctl daemon-reload

uninstall:
	sudo rm -f /usr/local/bin/c-hjkl /etc/systemd/system/c-hjkl.service
	sudo systemctl daemon-reload
