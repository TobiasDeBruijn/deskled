# Deskled
Control LED strips with Google Home

## Motivation
I've had 'dumb' LED strips under my desk for a couple years now. 
I didn't want to throw out these perfectly good strips to get something to work with Google Home.
These LED strips were controlled with a simple remote. So I went online to figure out what kind of LEDs
were used, `WS2811` was used. These LEDs can easily be controlled via a microcontroller, or in my case a Raspberry Pi Zero.

## Design
This software is only really build for 1 device. It does not support multiple devices connected to Google home, only the LED strip.
To persist data about the LED strip between software reboots I'm using MySQL. Now I know, MySQL is a little overkill, but hear me out.
Sqlite would've probably been the best option, but that requires crosscompiling libsqlite. I'm lazy, I don't want to do that.
The `mysql` crate is pure Rust, neat, makes it easy. The `mysql` crate depends on `flate2` for compression though, by default using zlib,
another C library I don't want. This feature was set in `refinery`, so a modified refinery is included in this repository that disables the `zlib` feature
and instead uses the `rust-backend` feature of zlib (Upstreaming this soon).

This repository also comes with a simple CLI, if you just want to use the terminal to control your lights. 

## Setup
Installing the software is a piece of cake. Clone the repo, have the Rust compiler, Make and git installed, and just run `make build-deskled`.
It'll take care of downloading the Pi cross compilation tools and crosscompile for the Pi Zero for you. Simple copy the binary to your Pi,
create `/etc/deskled/config.toml`:
```toml
[mysql]
host = 'your_mysql_host'
username = 'your_mysql_username'
password = 'your_mysql_password'
database = 'your_mysql_database_name'

[oauth2]
client_id = 'this_can_be_random'
client_secret = 'this_too_can_be_random'

[login]
username = 'username'
password = 'password'

[led]
# This is the amount of controllable sections
# Keep in mind that some led strips have multiple leds per controller,
# e.g. mine has 3 leds per controller. 
length = 30
```
You can then use systemd or whatever you  want to run the service. On your Pi you must also turn on SPI via `raspi-config`.
The server listens on port 8080, this must be available from the internet for Google to talk with it.

Configuring Google is pretty easy too. Look [here](https://developers.google.com/assistant/smarthome/develop/implement-oauth#configure_account_linking_in_the_console) to setup account linking (OAuth2).
The authorization URL path is `/oauth2/login`. The token URL path is `/oath2/exchange`. Then you want to configure your action, the fullfillment URL path is `/fulfillment`.
With that configured, you can use test mode and link with it from the Google Home app.

Some weird oddity: You wont see a color selection thing in the Google Home app on Android, for some reason Google deciced it's 
not necessary or something. Use Google Assistant to set the color instead. It's stupid, I know.

## Wiring
It's simple, like, really simple. Connect the ground of the LED strip with the Pi, a commong ground between the Pi, LED strip and your LED strips power supply is important.
Then connect the Data in of the led strip (commonly noted as `DIN`) with the `MOSI` (also known as `SPI_MOSI`) pin on the Pi. Thats pin 19.

Then power everything up, and it should work. If not, feel free to reach out. 

## License
Deskled is licensed under the Apache-2.0 or MIT license, at your discretion.

