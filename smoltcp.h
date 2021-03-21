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
extern smoltcp_stack_t *init_stack(void);

extern void destroy_stack(smoltcp_stack_t *stack);

// add_socket with default rx_buffer and tx_buffer size
extern uint8_t add_socket(smoltcp_stack_t *stack, uint8_t type);

// add_socket with custom rx_buffer and tx_buffer size
extern uint8_t add_socket_with_buffer(smoltcp_stack_t *stack, uint8_t type, uint8_t rx_buffer, uint8_t tx_buffer);

#endif