#ifndef DHT11_SENSOR_H
#define DHT11_SENSOR_H

#include "hardware/uart.h"
#include <stdio.h>
#include <stdlib.h>

#include "hardware/gpio.h"
#include "pico/time.h"


// datasheet: https://www.mouser.com/datasheet/2/758/DHT11-Technical-Data-Sheet-Translated-Version-1143054.pdf

typedef unsigned int uint;

class Dht11Sensor {
public:
  Dht11Sensor(uint gpNum);
  float getHumidity();
  float getTemperature();
  void update();

private:
  bool dht11Ready = false;

  float humidity = -1;
  float temperature = -1;
  float gpNum = 0;

  uint bits[40];

  bool checkReady();
  void sendWakeSignal();
  bool wakeResponse();
  bool wake();
  uint getBit(int index);
  uint getByte(int index);


  int waitChangeUs(bool currentState, int limit);

};










#endif
