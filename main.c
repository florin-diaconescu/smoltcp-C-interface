#include <uk/essentials.h>
/* Import user configuration: */
#include <uk/config.h>
#include <stdint.h>
#include <stdio.h>

#include "smoltcp.h"

int main(void)
{
    uint8_t ret;

    smoltcp_stack_t *stack = init_stack();

    ret = add_socket_with_buffer(stack, 0, 3500, 3500);
    if (ret) {
        printf("Wrong value! %d\n", ret);
    }
    else {
        printf("Good value! %d\n", ret);
    }

    destroy_stack(stack);
	return 0;
}
