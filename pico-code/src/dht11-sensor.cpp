#include "dht11-sensor.h"

//------public--------

Dht11Sensor::Dht11Sensor(uint gpNum)
{
  this->gpNum = gpNum;

  gpio_init(gpNum);
  gpio_set_dir(this->gpNum, GPIO_OUT);
  gpio_pull_up(gpNum);
  for(int i = 0; i < 40; i++)
    bits[i] = 0;
}

float Dht11Sensor::getHumidity()
{
  return this->humidity;
}

float Dht11Sensor::getTemperature()
{
  return this->temperature;
}


void Dht11Sensor::update()
{
  if(!dht11Ready)
  {
    checkReady();
    if(!dht11Ready)
      return;
    wake();
  }

  const int PACKET_NUM = 5;
  uint data[PACKET_NUM] {0, 0, 0, 0};
  uint check = 0;

  if(!wake())
  {
    uart_puts(uart0, "wake failed\n\r");
    return;
  }

  //expect low for begin bit reading
  waitChangeUs(true, 100);

  for(int i = 0; i < PACKET_NUM; i++)
  {
    data[i] = getByte(i);
    if(i < PACKET_NUM - 1)
      check+=data[i];
  }

  if(check == data[4])
  {
    this->humidity = (float)data[0] + ((float)data[1] / 10);
    this->temperature = (float)data[2] + ((float)data[3] / 10);
  }
  else
    uart_puts(uart0, "check failed\n\r");
}

//-------private----------

bool Dht11Sensor::checkReady()
{
  float elapsed = to_ms_since_boot(get_absolute_time());
  dht11Ready = elapsed > 1000;
  return dht11Ready;
}

void Dht11Sensor::sendWakeSignal()
{
  gpio_set_dir(gpNum, GPIO_OUT);
  gpio_pull_down(gpNum);
  sleep_ms(20);
  gpio_pull_up(gpNum);
  sleep_us(30);
}

bool Dht11Sensor::wakeResponse()
{
  gpio_set_dir(gpNum, GPIO_IN);

  int usElapsed = waitChangeUs(true, 100);
  if(usElapsed > 100)
    return 0;
  usElapsed = waitChangeUs(false, 100);
  if(usElapsed > 100)
    return 0;
  return 1;
}

bool Dht11Sensor::wake()
{
  sendWakeSignal();
  return wakeResponse();
}

uint Dht11Sensor::getBit(int index)
{
  int usElapsed = waitChangeUs(false, 100);
  usElapsed = waitChangeUs(true, 100);
//26-28us=0 , 70us=1
  return usElapsed > 40;
}

uint Dht11Sensor::getByte(int index)
{
  uint num = 0;
  for(int i = 0; i < 8; i++)
  {
    bits[index * 8 + i] = getBit(index);
    num = (num << 1) | bits[index * 8 + i];
  }
  return num;
}

int Dht11Sensor::waitChangeUs(bool currentState, int limit)
{
  int usElapsed = 0;
  while((bool)gpio_get(gpNum) == currentState)
  {
    sleep_us(1);
    usElapsed++;
    if(usElapsed > limit)
      break;
  }
  return usElapsed;
}
