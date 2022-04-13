#include "sensor_data_handler.h"


namespace SensorDataHandler {
//set externs
int sensorDataIndex = 0;
float currentTimeOffset = 0;
SensorData sensorData[MAX_RECORDS];
int currentReadingDelay = 30000;
critical_section modifyingDataHandlerGlobals;

//rx function globals
int readIndex;
const int READ_DATA_MAX = 100;
char readData[READ_DATA_MAX];

void handleArg();

void external_rx_handler()
{
    while (uart_is_readable(DATA_SEND_UART)) {
        uint8_t ch = uart_getc(DATA_SEND_UART);

        readData[readIndex] = ch;
        readIndex = (readIndex + 1) % READ_DATA_MAX;
        if(ch == '\n')
        {
          readData[readIndex] = '\0';
          handleArg();
          readIndex = 0;
        }
    }
}

void getCommand()
{
  critical_section_enter_blocking(&modifyingDataHandlerGlobals);

  int whenCalledSensorDataIndex = sensorDataIndex;

  critical_section_exit(&modifyingDataHandlerGlobals);

  if(whenCalledSensorDataIndex == 0)
  {
    uart_putc_raw(DATA_SEND_UART, (unsigned char)2); //empty
  }
  else
  {
    uart_putc_raw(DATA_SEND_UART, (unsigned char)1); //sending data

    for(int i = 0; i < whenCalledSensorDataIndex; i++)
    {
      uart_putc_raw(DATA_SEND_UART, sensorData[i].hmty[0]);
      uart_putc_raw(DATA_SEND_UART, sensorData[i].hmty[1]);
      uart_putc_raw(DATA_SEND_UART, sensorData[i].temp[0]);
      uart_putc_raw(DATA_SEND_UART, sensorData[i].temp[1]);
      uart_putc_raw(DATA_SEND_UART, sensorData[i].time[0]);
      uart_putc_raw(DATA_SEND_UART, sensorData[i].time[1]);
      uart_putc_raw(DATA_SEND_UART, sensorData[i].time[2]);

      //sync character
      if(i == whenCalledSensorDataIndex - 1)
        uart_putc_raw(DATA_SEND_UART, (unsigned char)254); //last
      else
        uart_putc_raw(DATA_SEND_UART, (unsigned char)255); //continue
    }

    critical_section_enter_blocking(&modifyingDataHandlerGlobals);

    sensorDataIndex = 0;

    currentTimeOffset = (float)to_ms_since_boot(get_absolute_time()) / 1000.0f;

    critical_section_exit(&modifyingDataHandlerGlobals);
  }
}

void delayCmd()
{
  uart_putc_raw(DATA_SEND_UART, (unsigned char)1);
  //get delay in seconds
  unsigned char delay = uart_getc(DATA_SEND_UART);
  if(delay == 0) //return delay if 0
    uart_putc_raw(DATA_SEND_UART, (unsigned char)(currentReadingDelay / 1000));
  else
  {
    critical_section_enter_blocking(&modifyingDataHandlerGlobals);
    currentReadingDelay = delay * 1000;
    critical_section_exit(&modifyingDataHandlerGlobals);
    //send back delay for confirm
    uart_putc_raw(DATA_SEND_UART, delay);
  }
}

bool sameCharArray(const char *command, char *check, int length)
{
  for(int i = 0; i < length; i++)
  {
    if(command[i] != check[i])
      return false;
  }
  return true;
}

void handleArg()
{
  const char GET_CMD[] = { 'g', 'e', 't' };
  const char DELAY_CMD[] = { 'd', 'e', 'l', 'a', 'y' };
  if(sameCharArray(GET_CMD, readData, 3))
    getCommand();
  else if(sameCharArray(DELAY_CMD, readData, 5))
    delayCmd();
  else
    uart_putc_raw(DATA_SEND_UART, (unsigned char)3); //unknown command
}


}//end namespace
