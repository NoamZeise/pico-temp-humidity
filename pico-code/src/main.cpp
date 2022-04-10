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


static int chars_rxed = 0;

void on_uart_rx() {
    while (uart_is_readable(uart1)) {
        uint8_t ch = uart_getc(uart1);
        // Can we send it back?
        if (uart_is_writable(uart1)) {
            // Change it slightly first!
            ch++;
            uart_putc(uart1, ch);
        }
        chars_rxed++;
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

    const char *msg = "Temp: %.1f Celsius\n\rHmty: %.1f Percent\n\r\n\r";
    int len = snprintf(NULL, 0, msg, tempHumSensor.getTemperature(), tempHumSensor.getHumidity());
    char *result = (char *)malloc(len + 1);
    snprintf(result, len + 1, msg, tempHumSensor.getTemperature(), tempHumSensor.getHumidity());
    bluetoothUart.print(result);
    free(result);

    gpio_put(PICO_DEFAULT_LED_PIN, 0);
  }
}
