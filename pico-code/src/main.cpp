#include "pico/stdlib.h"
#include "hardware/uart.h"
#include "hardware/irq.h"

#include <stdio.h>
#include <stdlib.h>

#include "uart_handle.h"
#include "dht11-sensor.h"


struct SensorData
{
    float temperature = -1;
    float humidity = -1;
};


static int readIndex = 0;
static const int READ_DATA_MAX = 100;
static char readData[READ_DATA_MAX];

static bool dataRequest = false;

void on_uart_rx() {
  const char getMsg[3] = {'g', 'e', 't'};

    while (uart_is_readable(uart1)) {
        uint8_t ch = uart_getc(uart1);
        if(readIndex > READ_DATA_MAX)
          readIndex = 0;

        readData[readIndex++] = ch;
        if(ch == '\r')
        {
          readData[readIndex] = '\0';
          if(readData[0] == getMsg[0] && readData[1] == getMsg[1] && readData[2] == getMsg[2])
          {
            dataRequest = true;
          }
          else
          {
            uart_puts(uart1, "unknown command!\n\r  get -> returns all stored readings\n\r\n\r");
          }
          readIndex = 0;
        }
    }
}

int main() {

  UartHandle passthroughUart(0, 1, uart0, 115200);
  UartHandle bluetoothUart(4, 5, uart1, 9600);

  uart_set_hw_flow(uart1, false, false);
  uart_set_fifo_enabled(uart1, false);
  irq_set_exclusive_handler(UART1_IRQ, on_uart_rx);
  irq_set_enabled(UART1_IRQ, true);

  uart_set_irq_enables(uart1, true, false);

  Dht11Sensor tempHumSensor(16);

  gpio_init(PICO_DEFAULT_LED_PIN);
  gpio_set_dir(PICO_DEFAULT_LED_PIN, GPIO_OUT);
  gpio_put(PICO_DEFAULT_LED_PIN, 0);

  const int MAX_RECORDS = 1000;
  SensorData sensorData[MAX_RECORDS];
  int sensorDataIndex = 0;


  while (true) {
    sleep_ms(5000);

    gpio_put(PICO_DEFAULT_LED_PIN, 1);

    tempHumSensor.update();

    sensorData[sensorDataIndex].temperature = tempHumSensor.getTemperature();
    sensorData[sensorDataIndex].humidity = tempHumSensor.getHumidity();
    sensorDataIndex = (sensorDataIndex + 1) % MAX_RECORDS;

    if(dataRequest)
    {
      for(int i = 0; i < sensorDataIndex; i++)
      {
        const char *msg = "Temp: %.1f Celsius    Hmty: %.1f Percent\n\r";
        int len = snprintf(NULL, 0, msg, sensorData[i].temperature, sensorData[i].humidity);
        char *result = (char *)malloc(len + 1);
        snprintf(result, len + 1, msg, sensorData[i].temperature, sensorData[i].humidity);
        bluetoothUart.print(result);
        free(result);
      }
      sensorDataIndex = 0;
      dataRequest = false;
    }

    sleep_ms(10);
    gpio_put(PICO_DEFAULT_LED_PIN, 0);
  }
}
