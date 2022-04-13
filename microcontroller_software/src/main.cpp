#include "pico/stdlib.h"
#include "pico/sync.h"
#include "hardware/uart.h"
#include "hardware/irq.h"


#include <stdio.h>
#include <stdlib.h>

#include "uart_handle.h"
#include "dht11-sensor.h"
#include "sensor_data_handler.h"

void get_sensor_reading(Dht11Sensor &sensor);

int main() {

  critical_section_init(&SensorDataHandler::modifyingDataHandlerGlobals);

  UartHandle passthroughUart(0, 1, uart0, 115200);
  UartHandle bluetoothUart(4, 5, DATA_SEND_UART, DATA_SEND_UART_BAUD);
  bluetoothUart.setRxHandler(SensorDataHandler::external_rx_handler);

  Dht11Sensor tempHumSensor(16);

  gpio_init(PICO_DEFAULT_LED_PIN);
  gpio_set_dir(PICO_DEFAULT_LED_PIN, GPIO_OUT);
  gpio_put(PICO_DEFAULT_LED_PIN, 0);

  while (true) {
    sleep_ms(SensorDataHandler::currentReadingDelay);
    gpio_put(PICO_DEFAULT_LED_PIN, 1);
    get_sensor_reading(tempHumSensor);
    gpio_put(PICO_DEFAULT_LED_PIN, 0);
  }
}

void get_sensor_reading(Dht11Sensor &tempHumSensor)
{
  tempHumSensor.update();

  unsigned char data[4] = {0, 0, 0, 0};
  tempHumSensor.get4u8Readings(data);

  critical_section_enter_blocking(&SensorDataHandler::modifyingDataHandlerGlobals);
  unsigned int timestamp = ((float)to_ms_since_boot(get_absolute_time()) / 1000.0f) - SensorDataHandler::currentTimeOffset;
  critical_section_exit(&SensorDataHandler::modifyingDataHandlerGlobals);

  SensorDataHandler::sensorData[SensorDataHandler::sensorDataIndex].hmty[0] = data[0];
  SensorDataHandler::sensorData[SensorDataHandler::sensorDataIndex].hmty[1] = data[1];
  SensorDataHandler::sensorData[SensorDataHandler::sensorDataIndex].temp[0] = data[2];
  SensorDataHandler::sensorData[SensorDataHandler::sensorDataIndex].temp[1] = data[3];
  SensorDataHandler::sensorData[SensorDataHandler::sensorDataIndex].time[0] = timestamp;
  SensorDataHandler::sensorData[SensorDataHandler::sensorDataIndex].time[1] = timestamp >> 8;
  SensorDataHandler::sensorData[SensorDataHandler::sensorDataIndex].time[2] = timestamp >> 16;

  critical_section_enter_blocking(&SensorDataHandler::modifyingDataHandlerGlobals);
  SensorDataHandler::sensorDataIndex = (SensorDataHandler::sensorDataIndex + 1) % SensorDataHandler::MAX_RECORDS;
  critical_section_exit(&SensorDataHandler::modifyingDataHandlerGlobals);
}
