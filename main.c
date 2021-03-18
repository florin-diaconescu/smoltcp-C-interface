#include <uk/essentials.h>
/* Import user configuration: */
#include <uk/config.h>
#include <stdint.h>

#include "smoltcp.h"

int main(void)
{
	Stack s;
	SmolSocket sock = add_socket(s, 0);
	return 0;
}
