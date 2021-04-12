#include <uk/essentials.h>
/* Import user configuration: */
#include <uk/config.h>
#include <stdint.h>
#include <stdio.h>
#include <fcntl.h>

#include "smoltcp.h"

int main(int argc, char *argv[])
{
    uint8_t client, server;
    int ret = 0;
    int done = 0, did_listen = 0, did_connect = 0;

    smoltcp_stack_t *lo_stack = init_loopback_stack();
    add_ipv4_address(lo_stack, 127, 0, 0, 1, 8);
    client = add_socket_with_buffer(lo_stack, TCP, 3500, 3500);
    server = add_socket_with_buffer(lo_stack, TCP, 3500, 3500);
//    add_ipv4_address(stack, 192, 168, 69, 1, 24);
//    add_ipv4_address(stack, 192, 168, 69, 2, 24);
//    add_ipv6_address(stack, 0xfdaa, 0, 0, 0, 0, 0, 0, 1, 64);
//    add_ethernet_address(stack, 0x02, 0x00, 0x00, 0x00, 0x00, 0x01);
	build_interface(lo_stack);

	while (1) {
        ret = poll_interface(lo_stack);
        if (ret == 1)
            break;
	}

    destroy_stack(lo_stack);
	return 0;
}
