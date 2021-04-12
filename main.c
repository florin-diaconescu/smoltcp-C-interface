#include <uk/essentials.h>
/* Import user configuration: */
#include <uk/config.h>
#include <stdint.h>
#include <stdio.h>
#include <fcntl.h>

#include "smoltcp.h"

#define SERVER_PORT 1234
#define CLIENT_PORT 65000

int main(int argc, char *argv[])
{
    uint8_t client, server;
    struct Ipv4AddressC lo_addr = {
            .ip_address = { 127, 0, 0, 1 }
    };

    smoltcp_stack_t *lo_stack = init_loopback_stack();
    add_ipv4_address(lo_stack, 127, 0, 0, 1, 8);
    client = add_socket_with_buffer(lo_stack, TCP, 3500, 3500);
    server = add_socket_with_buffer(lo_stack, TCP, 3500, 3500);
//    add_ipv4_address(stack, 192, 168, 69, 1, 24);
//    add_ipv4_address(stack, 192, 168, 69, 2, 24);
//    add_ipv6_address(stack, 0xfdaa, 0, 0, 0, 0, 0, 0, 1, 64);
//    add_ethernet_address(stack, 0x02, 0x00, 0x00, 0x00, 0x00, 0x01);
	build_interface(lo_stack);

//	while (1) {
//        ret = poll_interface(lo_stack);
//        if (ret == 1)
//            break;
//	}

    smoltcp_listen(lo_stack, lo_addr, server, SERVER_PORT);
    smoltcp_connect(lo_stack, lo_addr, SERVER_PORT, client, CLIENT_PORT);

    destroy_stack(lo_stack);
	return 0;
}
