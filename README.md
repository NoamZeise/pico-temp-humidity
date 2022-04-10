# Work in progress temperature and humidity collector project

using the [raspberry pi pico](https://www.raspberrypi.com/products/raspberry-pi-pico/) microcontroller board, [DHT11 sensor](https://components101.com/sensors/dht11-temperature-sensor), and [HC-05 bluetooth module](https://components101.com/wireless/hc-05-bluetooth-module).

Current State:
* reads temp and humidity readings with gpio from DHT11
* writes current temp and humidity with uart over bluetooth if sent "get"

Goal:
* upload to computer through a binary and save to file instead of using putty/minicom

## dependancies

I'm using the [pico SDK](https://www.raspberrypi.com/documentation/microcontrollers/c_sdk.html) which gives abstraction of the hardware and cmake build helper functions
