#include "pico/stdlib.h"
#include "pico/sync.h"
#include "hardware/uart.h"
#include "hardware/irq.h"


#include <stdio.h>
#include <stdlib.h>

#include "uart_handle.h"
#include "dht11-sensor.h"


struct SensorData
{
  unsigned char hmty[2];
  unsigned char temp[2];
  unsigned char time[3];
};

const int MAX_RECORDS = 10000;
static SensorData sensorData[MAX_RECORDS];
static int sensorDataIndex = 0;

static int readIndex = 0;
static float currentTimeOffset = 0;
static const int READ_DATA_MAX = 100;
static char readData[READ_DATA_MAX];

static mutex_t readIndexMutex;

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

            mutex_enter_blocking(&readIndexMutex);

            if(sensorDataIndex == 0)
            {
              uart_putc_raw(uart1, 'e'); //empty
            }
            else
            {
              uart_putc_raw(uart1, 'a'); //sending data

              for(int i = 0; i < sensorDataIndex; i++)
              {
                uart_putc_raw(uart1, sensorData[i].hmty[0]);
                uart_putc_raw(uart1, sensorData[i].hmty[1]);
                uart_putc_raw(uart1, sensorData[i].temp[0]);
                uart_putc_raw(uart1, sensorData[i].temp[1]);
                uart_putc_raw(uart1, sensorData[i].time[0]);
                uart_putc_raw(uart1, sensorData[i].time[1]);
                uart_putc_raw(uart1, sensorData[i].time[2]);
              }

              sensorDataIndex = 0;
              currentTimeOffset = to_ms_since_boot(get_absolute_time()) / 1000;
            }

            mutex_exit(&readIndexMutex);
          }
          else
          {
            uart_putc_raw(uart1, 'd'); //not sending data
          }
          readIndex = 0;
        }
    }
}

int main() {

  mutex_init(&readIndexMutex);

  UartHandle passthroughUart(0, 1, uart0, 115200);
  UartHandle bluetoothUart(4, 5, uart1, 9600);

//  uart_set_hw_flow(uart1, false, false);
//  uart_set_fifo_enabled(uart1, false);
  irq_set_exclusive_handler(UART1_IRQ, on_uart_rx);
  irq_set_enabled(UART1_IRQ, true);

  uart_set_irq_enables(uart1, true, false);

  Dht11Sensor tempHumSensor(16);

  gpio_init(PICO_DEFAULT_LED_PIN);
  gpio_set_dir(PICO_DEFAULT_LED_PIN, GPIO_OUT);
  gpio_put(PICO_DEFAULT_LED_PIN, 0);

  currentTimeOffset = 0;

  while (true) {
    sleep_ms(30000); //1 day = 2880 readings

    gpio_put(PICO_DEFAULT_LED_PIN, 1);

    tempHumSensor.update();

    mutex_enter_blocking(&readIndexMutex);

    unsigned char data[4] = {0, 0, 0, 0};
    tempHumSensor.get4u8Readings(data);
    unsigned int timestamp = to_ms_since_boot(get_absolute_time()) / 1000 - currentTimeOffset;

    sensorData[sensorDataIndex].hmty[0] = data[0];
    sensorData[sensorDataIndex].hmty[1] = data[1];
    sensorData[sensorDataIndex].temp[0] = data[2];
    sensorData[sensorDataIndex].temp[1] = data[3];
    sensorData[sensorDataIndex].time[0] = timestamp;
    sensorData[sensorDataIndex].time[1] = timestamp >> 8;
    sensorData[sensorDataIndex].time[2] = timestamp >> 16;

    sensorDataIndex = (sensorDataIndex + 1) % MAX_RECORDS;

    mutex_exit(&readIndexMutex);

    sleep_ms(10);
    gpio_put(PICO_DEFAULT_LED_PIN, 0);
  }
}
