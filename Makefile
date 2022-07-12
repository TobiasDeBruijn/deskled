.PHONY: build-cli cli deskled build-deskled

target/rpi-tools:
	git clone -q --depth=1 https://github.com/raspberrypi/tools.git ./target/rpi-tools

build-cli: target/rpi-tools
	cargo +nightly build -Zbuild-std --bin cli --target armv6-unknown-linux-gnueabihf.json --release

build-deskled: target/rpi-tools
	cargo +nightly build -Zbuild-std --bin deskled --target armv6-unknown-linux-gnueabihf.json --release

TARGET_IP=10.10.4.98
TARGET_USER=tobias

RUN_ARGS=""
cli: build-cli
	rsync target/armv6-unknown-linux-gnueabihf/release/cli ${TARGET_USER}@${TARGET_IP}:/tmp/deskled_cli
	ssh ${TARGET_USER}@${TARGET_IP} /tmp/deskled_cli ${RUN_ARGS}

deskled: build-deskled
	rsync target/armv6-unknown-linux-gnueabihf/release/deskled ${TARGET_USER}@${TARGET_IP}:/tmp/deskled

	ssh ${TARGET_USER}@${TARGET_IP} sudo systemctl stop deskled
	ssh ${TARGET_USER}@${TARGET_IP} sudo cp /tmp/deskled /usr/local/bin/
	ssh ${TARGET_USER}@${TARGET_IP} sudo chmod a+x /usr/local/bin/deskled
	ssh ${TARGET_USER}@${TARGET_IP} sudo systemctl start deskled
