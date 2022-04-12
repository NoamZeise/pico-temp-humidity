#include "uart_handle.h"

UartHandle::UartHandle(uint TXGP, uint RXGP, uart_inst_t* uartNum, uint baudRate)
{
  this->uartNum = uartNum;

  uart_init(uartNum, baudRate);
  gpio_set_function(TXGP, GPIO_FUNC_UART);
  gpio_set_function(RXGP, GPIO_FUNC_UART);

}


void UartHandle::print(const char* text)
{
  uart_puts(uartNum, text);
}

bool UartHandle::readable()
{
  return uart_is_readable(uartNum);
}

bool UartHandle::writable()
{
  return uart_is_writable(uartNum);
}

char UartHandle::getChar()
{
  return uart_getc(uart1);
}
