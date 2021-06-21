#include <uk/essentials.h>
/* Import user configuration: */
#include <uk/config.h>
#include <stdint.h>
#include <stdio.h>
#include <fcntl.h>
#include <uk/netdev.h>
#include <uk/alloc.h>
#include <assert.h>
#include <string.h>

#include "smoltcp.h"
// #include "eurosys_netdev.h"

#define UKNETDEV_BPS 1000000000u
#define UKNETDEV_BUFLEN 2048

#define ETH_PAD_SIZE 2

#define SERVER_PORT 1234
#define CLIENT_PORT 65000

#define UKNETDEV_MODE

struct uk_netdev *dev;
struct uk_netbuf *netbuf;

/* These headers are taken from linux */
struct	ether_header {
	uint8_t	ether_dhost[6];
	uint8_t	ether_shost[6];
	uint16_t ether_type;
}__attribute__((packed));

struct udphdr {
	uint16_t source;
	uint16_t dest;
	uint16_t len;
	uint16_t check;
}__attribute__((packed));

struct iphdr 
{
	uint8_t	ihl:4,
		version:4;
	uint8_t	tos;
	uint16_t	tot_len;
	uint16_t	id;
	uint16_t	frag_off;
	uint8_t	ttl;
	uint8_t	protocol;
	uint16_t	check;
	uint32_t	saddr;
	uint32_t	daddr;
}__attribute__((packed));

static uint16_t tx_headroom = ETH_PAD_SIZE;
static uint16_t rx_headroom = ETH_PAD_SIZE;

#define USE_SIMPLE_QUEUE
//#define USE_TX_BURST
//#define USE_RX_BURST
#define RX_ONLY_ONE_PACKET

struct uk_netbuf *alloc_netbuf(struct uk_alloc *a, size_t alloc_size,
		size_t headroom)
{
	void *allocation;
	struct uk_netbuf *b;

	allocation = uk_malloc(a, alloc_size);
	if (unlikely(!allocation))
		goto err_out;

	b = uk_netbuf_prepare_buf(allocation, alloc_size,
			headroom, 0, NULL);
	if (unlikely(!b)) {
		goto err_free_allocation;
	}

	b->_a = a;
	b->len = b->buflen - headroom;

	return b;

err_free_allocation:
	uk_free(a, allocation);
err_out:
	return NULL;
}

static uint16_t netif_alloc_rxpkts(void *argp, struct uk_netbuf *nb[],
		uint16_t count)
{
	struct uk_alloc *a;
	uint16_t i;

	UK_ASSERT(argp);

	a = (struct uk_alloc *) argp;

	for (i = 0; i < count; ++i) {
		nb[i] = alloc_netbuf(a, UKNETDEV_BUFLEN, rx_headroom);
		assert(nb[i]);
	}

	return i;
}

void print_ip(uint32_t ip)
{
    unsigned char bytes[4];
    bytes[0] = ip & 0xFF;
    bytes[1] = (ip >> 8) & 0xFF;
    bytes[2] = (ip >> 16) & 0xFF;
    bytes[3] = (ip >> 24) & 0xFF;
    fprintf(stderr, "%d.%d.%d.%d\n", bytes[0], bytes[1], bytes[2], bytes[3]);
}


static void inline prepare_packet(struct uk_netbuf *nb)
{

	struct ether_header *eth_header;
	struct iphdr *ip_hdr;
	struct udphdr *udp_hdr;


	eth_header = (struct ether_header *) nb->data;
	// IPv4 is encapsulated
	if (eth_header->ether_type == 8) {
		ip_hdr = (struct iphdr *)((char *)nb->data + sizeof(struct ether_header));

		// If IP protocol is UDP
		if (ip_hdr->protocol == 0x11) {
			ip_hdr = (struct iphdr *)((char *)nb->data + sizeof(struct ether_header));
			udp_hdr = (struct udphdr *)((char *)nb->data + sizeof(struct ether_header) + sizeof(struct iphdr));
			// fprintf(stderr, "\nshost: ");
			// for (int i = 0; i < 6; i++) {
			// 	fprintf(stderr, "%d ", eth_header->ether_shost[i]);
			// }
			// fprintf(stderr, "\ndhost: ");
			// for (int i = 0; i < 6; i++) {
			// 	fprintf(stderr, "%d ", eth_header->ether_dhost[i]);
			// }
			// fprintf(stderr, "\n");
			// print_ip(ip_hdr->saddr);
			// print_ip(ip_hdr->daddr);
			// fprintf(stderr, "saddr: %d daddr: %d\n", udp_hdr->source, udp_hdr->dest);
			/* Switch MAC */
			uint8_t tmp[6];
			memcpy(tmp, eth_header->ether_dhost, 6);
			memcpy(eth_header->ether_dhost, eth_header->ether_shost, 6);
			memcpy(eth_header->ether_shost, tmp, 6);

			/* Switch IP addresses */
			ip_hdr->saddr ^= ip_hdr->daddr;
			ip_hdr->daddr ^= ip_hdr->saddr;
			ip_hdr->saddr ^= ip_hdr->daddr;

			/* switch UDP PORTS */
			udp_hdr->source ^= udp_hdr->dest;
			udp_hdr->dest ^= udp_hdr->source;
			udp_hdr->source ^= udp_hdr->dest;

			/* No checksum requiered, they are 16 bits and
			 * switching them does not influence the checsum
			 * */
		}
	}
}

static inline void uknetdev_output(struct uk_netdev *dev, struct uk_netbuf *nb)
{
	int ret;

	do {
		ret = uk_netdev_tx_one(dev, 0, nb);
	} while(uk_netdev_status_notready(ret));

	if (ret < 0) {
		uk_netbuf_free_single(nb);
	}
}

void uknetdev_output_wrapper (void *new_data) {
	struct ether_header *eth_header;
	struct iphdr *ip_hdr;
	struct udphdr *udp_hdr;

	eth_header = (struct ether_header *) netbuf->data;
	// IPv4 is encapsulated
	if (eth_header->ether_type == 8) {
		ip_hdr = (struct iphdr *)((char *)netbuf->data + sizeof(struct ether_header));

		// If IP protocol is UDP
		if (ip_hdr->protocol == 0x11) {
			ip_hdr = (struct iphdr *)((char *)netbuf->data + sizeof(struct ether_header));
			udp_hdr = (struct udphdr *)((char *)netbuf->data + sizeof(struct ether_header) + sizeof(struct iphdr));
			fprintf(stderr, "INAINTE !!! %d %d\n", udp_hdr->source, udp_hdr->dest);
		}
	}
	memcpy(netbuf->data, new_data, sizeof(new_data));

	eth_header = (struct ether_header *) new_data;
	// IPv4 is encapsulated
	//if (eth_header->ether_type == 8) {
		ip_hdr = (struct iphdr *)((char *)netbuf->data + sizeof(struct ether_header));

		// If IP protocol is UDP
		if (ip_hdr->protocol == 0x11) {
			ip_hdr = (struct iphdr *)((char *)netbuf->data + sizeof(struct ether_header));
			udp_hdr = (struct udphdr *)((char *)netbuf->data + sizeof(struct ether_header) + sizeof(struct iphdr));
			fprintf(stderr, "DUPA !!! %d %d\n", udp_hdr->source, udp_hdr->dest);
		}
	//}
	else fprintf(stderr, "DUPA !!! %d\n", eth_header->ether_type);
	uknetdev_output(dev, netbuf);
}

static inline void* packet_handler(struct uk_netdev *dev,
		uint16_t queue_id __unused, void *argp)
{

	struct ether_header *eth_header;
	struct iphdr *ip_hdr;
	int ret;
	struct uk_netbuf *nb;

back:
    ret = uk_netdev_rx_one(dev, 0, &nb);

    if (uk_netdev_status_notready(ret)) {
        goto back;
    }

    netbuf = nb;

	return nb->data;
}

void* packet_handler_wrapper(void) {
    void *nb;
	nb = packet_handler(dev, 0, NULL);

	return nb;
}

int main(int argc, char *argv[])
{
    uint8_t client, server;
    int did_connect = 0, did_listen = 0, did_send = 0, did_recv = 0;
    struct Ipv4AddressC lo_addr = {
            .ip_address = { 127, 0, 0, 1 }
    };

    struct uk_alloc *a;

	struct uk_netdev_conf dev_conf;
	struct uk_netdev_rxqueue_conf rxq_conf;
	struct uk_netdev_txqueue_conf txq_conf;
	int devid = 0;
	int ret, i;

	/* Get pointer to default UK allocator */
	a = uk_alloc_get_default();
	assert(a != NULL);

	dev = uk_netdev_get(devid);
	assert(dev != NULL);

	struct uk_netdev_info info;
	uk_netdev_info_get(dev, &info);
	assert(info.max_tx_queues);
	assert(info.max_rx_queues);

	assert(uk_netdev_state_get(dev) == UK_NETDEV_UNCONFIGURED);

	rx_headroom = (rx_headroom < info.nb_encap_rx)
		? info.nb_encap_rx : rx_headroom;
	tx_headroom = (tx_headroom < info.nb_encap_tx)
		? info.nb_encap_tx : tx_headroom;

	dev_conf.nb_rx_queues = 1;
	dev_conf.nb_tx_queues = 1;

	/* Configure the device */
	ret = uk_netdev_configure(dev, &dev_conf);
	assert(ret >= 0);

	/* Configure the RX queue */
	rxq_conf.a = a;
	rxq_conf.alloc_rxpkts = netif_alloc_rxpkts;
	rxq_conf.alloc_rxpkts_argp = a;

	//printf("Running busy waiting\n");
	rxq_conf.callback = NULL;
	rxq_conf.callback_cookie = NULL;

	ret = uk_netdev_rxq_configure(dev, 0, 0, &rxq_conf);
	assert(ret >= 0);

	/*  Configure the TX queue*/
	txq_conf.a = a;
	ret = uk_netdev_txq_configure(dev, 0, 0, &txq_conf);
	assert(ret >= 0);

	/* GET mTU */
	uint16_t mtu = uk_netdev_mtu_get(dev);
	assert(mtu == 1500);

	/* Start the netdev */
	ret = uk_netdev_start(dev);

	/* No interrupts */
	ret = uk_netdev_rxq_intr_disable(dev, 0);
	assert(ret >= 0);



#ifdef LOOPBACK_MODE
    smoltcp_stack_t *lo_stack = init_loopback_stack();
    add_ipv4_address(lo_stack, 127, 0, 0, 1, 8);
    client = add_socket_with_buffer(lo_stack, TCP, 3500, 3500);
    server = add_socket_with_buffer(lo_stack, TCP, 3500, 3500);
	fprintf(stderr, "C code: ether_header %d iphdr %d udphdr %d", sizeof(struct ether_header), sizeof(struct iphdr), sizeof(struct udphdr));
//    add_ipv4_address(stack, 192, 168, 69, 1, 24);
//    add_ipv4_address(stack, 192, 168, 69, 2, 24);
//    add_ipv6_address(stack, 0xfdaa, 0, 0, 0, 0, 0, 0, 1, 64);
//    add_ethernet_address(stack, 0x02, 0x00, 0x00, 0x00, 0x00, 0x01);
	build_interface(lo_stack);
#endif /* LOOPBACK_MODE */

#ifdef UKNETDEV_MODE
	smoltcp_stack_t *uk_stack = init_uknetdev_stack();
	add_ipv4_address(uk_stack, 127, 0, 0, 1, 8);
	/* Set Unikraft Ethernet address */
	const struct uk_hwaddr *hwaddr;
	uint8_t hw[6];

	hwaddr = uk_netdev_hwaddr_get(dev);
	for (i = 0; i < 6; i++) {
		fprintf(stderr, "%u ", hwaddr->addr_bytes[i]);
	}
	fprintf(stderr, "\n \n \n");

	add_ethernet_address(uk_stack, 0x52, 0x55, 0x00, 0xd1, 0x55, 0x01);
    server = add_socket_with_buffer(uk_stack, UDP, 3500, 3500);
	build_interface(uk_stack);

    smoltcp_bind(uk_stack, client, 4321);

	while (1) {
		ret = smoltcp_uk_recv(uk_stack, server);
		fprintf(stderr, "Handler done!");
        smoltcp_uk_send(uk_stack, netbuf->data);
        fprintf(stderr, "Uknetdev_outputed!");
	}

	destroy_stack(uk_stack);

#endif /* UKNETDEV_MODE */

#ifdef LOOPBACK_MODE
	/* Loopback code */
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

#endif /* LOOPBACK_MODE */

	return 0;
}
