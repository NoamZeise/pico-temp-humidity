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
  unsigned char hmty[2] = {0, 0};
  unsigned char temp[2] = {0, 0};
  unsigned char time[3] = {0, 0, 0};
};

const int MAX_RECORDS = 10000;
static SensorData sensorData[MAX_RECORDS];
static int sensorDataIndex = 0;

static int readIndex = 0;
static float currentTimeOffset = 0;
static const int READ_DATA_MAX = 100;
static char readData[READ_DATA_MAX];

static critical_section changingIndexCritSec;

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
          uart_puts(uart0, "got end\n\rread:\n\r"); //empty
          uart_puts(uart0, readData); //empty
          if(readData[0] == getMsg[0] && readData[1] == getMsg[1] && readData[2] == getMsg[2])
          {
            uart_puts(uart0, "getting data \n\r"); //empty

            critical_section_enter_blocking(&changingIndexCritSec);

            int whenCalledSensorDataIndex = sensorDataIndex;

            critical_section_exit(&changingIndexCritSec);

            if(whenCalledSensorDataIndex == 0)
            {
              uart_puts(uart0, "empty\n\r"); //empty
              uart_putc_raw(uart1, (unsigned char)2); //empty
            }
            else
            {
              uart_putc_raw(uart1, (unsigned char)1); //sending data

              for(int i = 0; i < whenCalledSensorDataIndex; i++)
              {
                uart_putc_raw(uart1, sensorData[i].hmty[0]);
                uart_putc_raw(uart1, sensorData[i].hmty[1]);
                uart_putc_raw(uart1, sensorData[i].temp[0]);
                uart_putc_raw(uart1, sensorData[i].temp[1]);
                uart_putc_raw(uart1, sensorData[i].time[0]);
                uart_putc_raw(uart1, sensorData[i].time[1]);
                uart_putc_raw(uart1, sensorData[i].time[2]);
              }

              critical_section_enter_blocking(&changingIndexCritSec);

              sensorDataIndex = 0;

              currentTimeOffset = (float)to_ms_since_boot(get_absolute_time()) / 1000.0f;

              critical_section_exit(&changingIndexCritSec);

              uart_puts(uart0, "get data end\n\r"); //empty
            }
          }
          else
          {
            uart_puts(uart0, "unknown command\n\r");
            uart_putc_raw(uart1, (unsigned char)3); //not sending data
          }
          readIndex = 0;
        }
    }
}

int main() {

  critical_section_init(&changingIndexCritSec);

  UartHandle passthroughUart(0, 1, uart0, 115200);
  UartHandle bluetoothUart(4, 5, uart1, 9600);

  uart_set_hw_flow(uart1, false, false);
  uart_set_format(uart1, 8, 1, UART_PARITY_NONE);

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

    unsigned char data[4] = {0, 0, 0, 0};
    tempHumSensor.get4u8Readings(data);
    critical_section_enter_blocking(&changingIndexCritSec);
    unsigned int timestamp = ((float)to_ms_since_boot(get_absolute_time()) / 1000.0f) - currentTimeOffset;
    critical_section_exit(&changingIndexCritSec);

    sensorData[sensorDataIndex].hmty[0] = data[0];
    sensorData[sensorDataIndex].hmty[1] = data[1];
    sensorData[sensorDataIndex].temp[0] = data[2];
    sensorData[sensorDataIndex].temp[1] = data[3];
    sensorData[sensorDataIndex].time[0] = timestamp;
    sensorData[sensorDataIndex].time[1] = timestamp >> 8;
    sensorData[sensorDataIndex].time[2] = timestamp >> 16;

    critical_section_enter_blocking(&changingIndexCritSec);

    sensorDataIndex = (sensorDataIndex + 1) % MAX_RECORDS;

    critical_section_exit(&changingIndexCritSec);

    sleep_ms(10);
    gpio_put(PICO_DEFAULT_LED_PIN, 0);
  }
}
