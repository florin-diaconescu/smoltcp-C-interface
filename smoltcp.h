#ifndef SMOLTCP_H
#define SMOLTCP_H

#include <stdint.h>
#define TCP 0
#define UDP 1

// structure definitions
typedef struct Ipv4AddressC {
    uint8_t ip_address[4];
} Ipv4AddressC;

struct Ipv4CidrC {
    Ipv4AddressC ip_address;
    uint32_t mask;
};

struct Ipv6AddressC {
    uint16_t ip_address[8];
};

typedef struct SmolSocket {
    uint8_t sockfd;
} SmolSocket;

typedef struct smoltcp_stack smoltcp_stack_t;

// function definitions
extern smoltcp_stack_t *init_tap_stack(char *interface_name);

extern smoltcp_stack_t *init_loopback_stack(void);

extern smoltcp_stack_t *init_uknetdev_stack(void);

extern void destroy_stack(smoltcp_stack_t *stack);

// add_socket with default rx_buffer and tx_buffer size
extern uint8_t add_socket(smoltcp_stack_t *stack, uint16_t type);

// add_socket with custom rx_buffer and tx_buffer size
extern uint8_t add_socket_with_buffer(smoltcp_stack_t *stack, uint16_t type,
                                      uint16_t rx_buffer, uint16_t tx_buffer);

// add ipv4 address
extern uint8_t add_ipv4_address(smoltcp_stack_t *stack, uint8_t a0, uint8_t a1,
                                uint8_t a2, uint8_t a3, uint8_t netmask);

// add ipv6 address
extern uint8_t add_ipv6_address(smoltcp_stack_t *stack, uint8_t a0, uint8_t a1,
                                uint8_t a2, uint8_t a3, uint8_t a4, uint8_t a5,
                                uint8_t a6, uint8_t a7, uint8_t netmask);

extern uint8_t add_ethernet_address(smoltcp_stack_t *stack, uint8_t a0, uint8_t a1,
                                    uint8_t a2, uint8_t a3, uint8_t a4, uint8_t a5);

extern uint8_t build_interface(smoltcp_stack_t *stack);

extern uint8_t poll_interface(smoltcp_stack_t *stack);

extern uint8_t smoltcp_listen(smoltcp_stack_t *stack, Ipv4AddressC server_ip, uint8_t socket, uint16_t port);

extern uint8_t smoltcp_connect(smoltcp_stack_t *stack, Ipv4AddressC server_ip,
                               uint16_t server_port, uint8_t socket, uint16_t client_port);

extern uint8_t smoltcp_send(smoltcp_stack_t *stack, uint8_t socket, char *message);

extern uint8_t smoltcp_uk_send(smoltcp_stack_t *stack, void *message);

extern uint8_t smoltcp_recv(smoltcp_stack_t *stack, uint8_t socket);

extern uint8_t smoltcp_uk_recv(smoltcp_stack_t *stack);

extern uint8_t smoltcp_close(smoltcp_stack_t *stack, uint8_t socket);

extern uint8_t uknetdev_input(smoltcp_stack_t *stack, void *packet);
#endif
