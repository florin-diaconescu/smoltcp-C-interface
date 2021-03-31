#include <uk/essentials.h>
/* Import user configuration: */
#include <uk/config.h>
#include <stdint.h>
#include <stdio.h>
#include <fcntl.h>

#include "smoltcp.h"

int main(int argc, char *argv[])
{
    uint8_t sock1, sock2;

    smoltcp_stack_t *lo_stack = init_loopback_stack();
    add_ipv4_address(lo_stack, 127, 0, 0, 1, 8);
    sock1 = add_socket_with_buffer(lo_stack, TCP, 3500, 3500);
    sock2 = add_socket_with_buffer(lo_stack, TCP, 3500, 3500);
//    add_ipv4_address(stack, 192, 168, 69, 1, 24);
//    add_ipv4_address(stack, 192, 168, 69, 2, 24);
//    add_ipv6_address(stack, 0xfdaa, 0, 0, 0, 0, 0, 0, 1, 64);
//    add_ethernet_address(stack, 0x02, 0x00, 0x00, 0x00, 0x00, 0x01);

    destroy_stack(lo_stack);
	return 0;
}
