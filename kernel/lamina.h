#ifndef _LAMINA_H
#define _LAMINA_H

#include <linux/types.h>
#include <linux/sched.h>

#define TARGET_CPU 0
#define LAMINA_CMD_WRITECTL 0x00001000

struct lamina_msg {
	__u64 ctl[6];
};

#endif // _LAMINA_H
