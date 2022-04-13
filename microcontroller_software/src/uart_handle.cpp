#include "uart_handle.h"

UartHandle::UartHandle(uint TXGP, uint RXGP, uart_inst_t* uartNum, uint baudRate)
{
  this->uartNum = uartNum;

  uart_init(uartNum, baudRate);
  gpio_set_function(TXGP, GPIO_FUNC_UART);
  gpio_set_function(RXGP, GPIO_FUNC_UART);

}
void UartHandle::setRxHandler(irq_handler_t handler)
{
  uart_set_hw_flow(this->uartNum, false, false);
  uart_set_format(this->uartNum, 8, 1, UART_PARITY_NONE);

  int uartIrq = uart_get_index(this->uartNum) == 0 ? UART0_IRQ : UART1_IRQ;

  irq_set_exclusive_handler(uartIrq, handler);
  irq_set_enabled(uartIrq, true);

  hasRxHandler = true;
  uart_set_irq_enables(this->uartNum, hasRxHandler, hasTxHandler);
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
