#ifndef DHT11_SENSOR_H
#define DHT11_SENSOR_H

#include "hardware/uart.h"
#include <stdio.h>
#include <stdlib.h>

#include "hardware/gpio.h"
#include "pico/time.h"


// datasheet: https://www.mouser.com/datasheet/2/758/DHT11-Technical-Data-Sheet-Translated-Version-1143054.pdf

class Dht11Sensor {
public:
  Dht11Sensor(uint gpNum);
  void get4u8Readings(unsigned char* buff);
  void update();

private:
  bool dht11Ready = false;

  unsigned char readings[4] = {0, 0, 0, 0};
  float gpNum = 0;

  unsigned char bits[40];

  bool checkReady();
  void sendWakeSignal();
  bool wakeResponse();
  bool wake();
  unsigned char getBit(int index);
  unsigned char getByte(int index);


  int waitChangeUs(bool currentState, int limit);

};










#endif
