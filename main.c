#include <uk/essentials.h>
/* Import user configuration: */
#include <uk/config.h>
#include <stdint.h>
#include <stdio.h>
#include <fcntl.h>

#include "smoltcp.h"

int main(void)
{
    uint8_t sock1, sock2;

    smoltcp_stack_t *stack = init_stack();

    sock1 = add_socket_with_buffer(stack, TCP, 3500, 3500);
    sock2 = add_socket_with_buffer(stack, TCP, 3500, 3500);
    add_ipv4_address(stack, 192, 168, 69, 1, 24);
    add_ipv4_address(stack, 192, 168, 69, 2, 24);
    add_ipv6_address(stack, 0xfdaa, 0, 0, 0, 0, 0, 0, 1, 64);

    destroy_stack(stack);
	return 0;
}
