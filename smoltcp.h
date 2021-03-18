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

typedef struct Stack {

} Stack;

typedef struct SmolSocket {
    uint8_t sockfd;
} SmolSocket;

// function definitions
extern SmolSocket add_socket(Stack stack, uint8_t type);

#endif