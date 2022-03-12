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
	u64 tmp[6];

	//pr_info("lamina: %016llx %016llx %016llx %016llx %016llx %016llx\n",
	//		msg->ctl[0], msg->ctl[1], msg->ctl[2], 
	//		msg->ctl[3], msg->ctl[4], msg->ctl[5]);

	// Haven't decided exactly how I want to do this yet:
	//	- Clear the enable bit on all PERF_CTL registers
	//	- Clear all of the PERF_CTR registers
	//	- Write the new set of PERF_CTL registers

	// Read PERF_CTL
	rdmsrl(0xc0010200, tmp[0]);
	rdmsrl(0xc0010202, tmp[1]);
	rdmsrl(0xc0010204, tmp[2]);
	rdmsrl(0xc0010206, tmp[3]);
	rdmsrl(0xc0010208, tmp[4]);
	rdmsrl(0xc001020a, tmp[5]);

	// Clear enable bit
	wrmsrl(0xc0010200, tmp[0] & ~(1 << 22));
	wrmsrl(0xc0010202, tmp[1] & ~(1 << 22));
	wrmsrl(0xc0010204, tmp[2] & ~(1 << 22));
	wrmsrl(0xc0010206, tmp[3] & ~(1 << 22));
	wrmsrl(0xc0010208, tmp[4] & ~(1 << 22));
	wrmsrl(0xc001020a, tmp[5] & ~(1 << 22));

	// Clear PERF_CTR
	wrmsrl(0xc0010201, 0);
	wrmsrl(0xc0010203, 0);
	wrmsrl(0xc0010205, 0);
	wrmsrl(0xc0010207, 0);
	wrmsrl(0xc0010209, 0);
	wrmsrl(0xc001020b, 0);

	// Write PERF_CTL
	wrmsrl(0xc0010200, msg->ctl[0]);
	wrmsrl(0xc0010202, msg->ctl[1]);
	wrmsrl(0xc0010204, msg->ctl[2]);
	wrmsrl(0xc0010206, msg->ctl[3]);
	wrmsrl(0xc0010208, msg->ctl[4]);
	wrmsrl(0xc001020a, msg->ctl[5]);

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

