#ifndef UART_HANDLE_H
#define UART_HANDLE_H

#include "hardware/gpio.h"
#include "hardware/uart.h"
#include "hardware/irq.h"

typedef unsigned int uint;

class UartHandle {
public:
  UartHandle(uint TXGP, uint RXGP, uart_inst_t* uartNum, uint baudRate);
  void setRxHandler(irq_handler_t handler);
  void print(const char* text);
  bool readable();
  bool writable();
  char getChar();
private:
  uart_inst_t* uartNum;
  bool hasRxHandler = false;
  bool hasTxHandler = false;
};


#endif
