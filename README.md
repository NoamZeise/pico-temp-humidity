# Work in progress temperature and humidity collector project

using the [raspberry pi pico](https://www.raspberrypi.com/products/raspberry-pi-pico/) microcontroller board, [DHT11 sensor](https://components101.com/sensors/dht11-temperature-sensor), and [HC-05 bluetooth module](https://components101.com/wireless/hc-05-bluetooth-module).

Current State:
* reads temp/humidity from DHT11 with gpio every 30 seconds
* saves readings + time in a large buffer
* when sent a request from the command line tool over bluetooth serial, sends the data to the PC, clears the buffer and resets the time
* PC tool saves the recieved data as a csv file on the user's machine as Humidity,Temperature,Time,

## dependancies

* the [pico SDK](https://www.raspberrypi.com/documentation/microcontrollers/c_sdk.html) which gives abstraction of the hardware and cmake build helper functions. The path needs to be set in CMakeLists.txt.
* The [C/C++ embedded toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm) for arm-cortex-m processors
* [Rustc and cargo](https://www.rust-lang.org/tools/install) for the command line tool
