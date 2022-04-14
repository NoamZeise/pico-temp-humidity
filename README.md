# Temperature and Humidity Collector
Software for the raspberry pi pico that collects the temperature and humidity at an interval and stores it into a large buffer, and a command line tool to get the stored data from the pico over serial uart, and get/set the pico's recording interval.

## Setup
[full setup with images can be found on my website](https://noamzeise.wordpress.com/2022/04/13/temperature-and-humidity-collector-command-line-tool/)

### pico setup
1. Download a release of this project (pico .uf2 file + cli executable for your OS), or build the binaries yourself.
2. Hold down the BOOTSEL button on the pico and plug it into your PC over usb. The flash storage of your pico should appear. 
3. Release the button and drag the .uf2 file onto the pico. The flash storage should close.
4. Connect the DHT11 to the pico (DHT11 pin to Pico pin):
    * Data    ->   GP16
    * VCC     ->   VSYS
    * GND     ->   GND
5. Connect the HC-05 to the pico (HC-05 pin to Pico pin):
    * TXD     ->   GP5
    * RXD     ->   GP4
    * VCC     ->   VSYS
    * GND     ->   GND
6. Connect the power supply (or you can skip this if you'll use the usb to power it)
    * Positve Terminal -> VSYS
    * Negative Terminal -> GND

### cli use

#### port

 Ensure the pico is on, pair with the HC-05 bluetooth module and note it's port
 
 * if you are on windows, the device manager will show your available ports under Ports (the format is COM[X]), you can either try them all with the command line tool, or connect and disconnect the pico to see which port appears and disappears.

* On linux you can pair with the device using bluez, then link it’s MAC address to an rfcomm port, so the port format would be /dev/rfcomm[x]. or use the dmesg | grep tty to output a history of devices connecting 

#### commands

* Get data from the pico with the "get [port] [file] [optional args]" command. the --useprev arg uses a previous record in the given file as an offset, the --useoffset [offset] arg uses a custom offset
* Get the pico's current sensor reading delay with "delay [port]", or pass the --set [delay] arg to set the pico to a delay between 1-255 seconds

## Process
* reads temp/humidity from DHT11 with gpio after each delay
* saves readings + time in a buffer (30,000 readings limit)
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
