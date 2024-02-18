/*
 * Since the rust ioctl bindings don't have all the structures and constants,
 * it's easier to just write the thing in C and link it in.
 */

#include <assert.h>
#include <stdint.h>
#include <string.h>
#include <sys/ioctl.h>
#include <sys/socket.h>

#ifdef __linux__
# include <linux/if.h>
# include <linux/if_tun.h>
#else
# include <net/if.h>
# define IFF_TUN 0x0001
# define IFF_TAP 0x0002
# define IFF_NO_PI 0x1000
# define TUNSETIFF _IOW('T', 202, int)
#endif

/**
 * fd
 * 	‒ the fd to turn into TUN or TAP.
 * 
 * name
 * 	‒ the name to use. If empty, kernel will assign something by itself.
 *   Must be buffer with capacity at least 33.
 * 
 * mode
 * 	‒ 1 = TUN, 2 = TAP.
 * 
 * packet_info
 * 	‒ if packet info should be provided, if the given value is 0 it will not prepend packet info.
 */
int tuntap_setup(int fd, unsigned char *name, int mode, int packet_info)
{
	struct ifreq	ifr;
	int				ioresult;

	memset(&ifr, 0, sizeof(ifr));
	if (mode == 1)
		ifr.ifr_flags = IFF_TUN;
	else if (mode == 2)
		ifr.ifr_flags = IFF_TAP;
	else
		assert(0);
	if (!packet_info)
	{
		ifr.ifr_flags |= IFF_NO_PI;
	}
	strncpy(ifr.ifr_name, (char *)name, IFNAMSIZ - 1);
	ioresult = ioctl(fd, TUNSETIFF, &ifr);
	if (ioresult < 0)
		return (ioresult);
	strncpy((char *)name, ifr.ifr_name, IFNAMSIZ < 32 ? IFNAMSIZ : 32);
	name[32] = '\0';
	return (0);
}
