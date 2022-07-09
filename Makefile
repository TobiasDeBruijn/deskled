.PHONY: build-cli cli

target/rpi-tools:
	git clone -q --depth=1 https://github.com/raspberrypi/tools.git ./target/rpi-tools

build-cli: target/rpi-tools
	cargo +nightly build -Zbuild-std --target armv6-unknown-linux-gnueabihf.json

TARGET_IP=10.10.200.103
TARGET_USER=tobias
RUN_ARGS=""
cli: build-cli
	rsync target/armv6-unknown-linux-gnueabihf/debug/cli ${TARGET_USER}@${TARGET_IP}:/tmp/deskled_cli
	ssh ${TARGET_USER}@${TARGET_IP} /tmp/deskled_cli ${RUN_ARGS}