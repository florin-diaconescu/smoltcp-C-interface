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
    int did_connect = 0, did_listen = 0, did_send = 0, did_recv = 0;
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

    while (1) {
        poll_interface(lo_stack);
        // server part
        if (!did_listen) {
            smoltcp_listen(lo_stack, lo_addr, server, SERVER_PORT);
            did_listen = 1;
        }
        else if (!did_recv) {
            if (smoltcp_recv(lo_stack, server) == 0 ) {
                smoltcp_close(lo_stack, server);
                did_recv = 1;
                break;
            }
        }
        // client part
        if (!did_connect) {
            smoltcp_connect(lo_stack, lo_addr, SERVER_PORT, client, CLIENT_PORT);
            did_connect = 1;
        }
        else {
            if (!did_send) {
                if (smoltcp_send(lo_stack, client, "test_buffer") == 0) {
                    smoltcp_close(lo_stack, client);
                    did_send = 1;
                }
            }
        }
    }

    destroy_stack(lo_stack);
	return 0;
}
