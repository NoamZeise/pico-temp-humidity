# Temperature and Humidity Collector
Software for the raspberry pi pico that collects the temperature and humidity at an interval and stores it into a large buffer, and a command line tool to get the stored data from the pico over serial uart, and get/set the pico's recording interval.


## Process
* reads temp/humidity from DHT11 with gpio every 30 seconds
* saves readings + time in a large buffer
* when sent a get request from the command line tool over bluetooth serial, sends the data to the PC, clears the buffer and resets the time
* PC tool saves the recieved data as a csv file (or appends to preexisting) on the user's machine as Time,Humidity,Temperature,
* the user can specify to use the previous reading's timestamp as an offset for the next ones, or a custom offset, or both
* The user can send a delay command to get the current recording delay, and a delay set command to set the delay

## Components

* [raspberry pi pico](https://www.raspberrypi.com/products/raspberry-pi-pico/) microcontroller board 
* [DHT11 sensor](https://components101.com/sensors/dht11-temperature-sensor) (might also work with DHT-22 as they have the same data format)
* [HC-05 bluetooth module](https://components101.com/wireless/hc-05-bluetooth-module) (or any other 9600 baud rate uart device connected to the pico's uart1 pins)

## Dependancies

* the [pico SDK](https://www.raspberrypi.com/documentation/microcontrollers/c_sdk.html) which gives abstraction of the hardware, as well as cmake helper functions. The path needs to be set in CMakeLists.txt.
* The [C/C++ embedded toolchain](https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm) for arm-cortex-m processors
* [Rustc and cargo](https://www.rust-lang.org/tools/install) and the [serial port](https://crates.io/crates/serialport) crate for the command line tool
