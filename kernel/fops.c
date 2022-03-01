// SPDX-License-Identifier: GPL-2.0

#include <linux/smp.h>
#include <linux/fs.h>
#include <asm/msr.h>

#include "lamina.h"
#include "fops.h"

struct lamina_msg msg;

void write_pmcs(void* info)
{
	struct lamina_msg *msg = info;

	pr_info("lamina: %016llx %016llx %016llx %016llx %016llx %016llx\n",
			msg->ctl[0], msg->ctl[1], msg->ctl[2], 
			msg->ctl[3], msg->ctl[4], msg->ctl[5]);

	//wrmsrl(0xc0010200, msg->ctl[0]);
	//wrmsrl(0xc0010202, msg->ctl[1]);
	//wrmsrl(0xc0010204, msg->ctl[2]);
	//wrmsrl(0xc0010206, msg->ctl[3]);
	//wrmsrl(0xc0010208, msg->ctl[4]);
	//wrmsrl(0xc001020a, msg->ctl[5]);

	return;
}

long int lamina_ioctl(struct file *file, unsigned int cmd, unsigned long arg)
{
	long int res = -EINVAL;

	switch (cmd) {
	case LAMINA_CMD_WRITECTL:
		res = copy_from_user(&msg, (struct lamina_msg *)arg, 
				sizeof(struct lamina_msg)
		);
		smp_call_function_single(TARGET_CPU, write_pmcs, (void*)&msg, true);
		break;
	default:
		break;
	}

	return res;
}

