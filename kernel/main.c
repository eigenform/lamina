// SPDX-License-Identifier: GPL-2.0

#include <linux/module.h>
#include <linux/moduleparam.h>
#include <linux/debugfs.h>
#include <linux/miscdevice.h>
#include <asm/processor-flags.h>
#include <asm/msr.h>

#include "lamina.h"
#include "fops.h"


static const struct file_operations lamina_fops = {
	.owner				= THIS_MODULE,
	.unlocked_ioctl		= lamina_ioctl,
};

static struct miscdevice lamina_dev = {
	.minor				= MISC_DYNAMIC_MINOR,
	.name				= "lamina",
	.fops				= &lamina_fops,
	.mode				= 0666,
};

static u32 PERF_CTL_MSR[6] = {
	0xc0010200, 0xc0010202, 0xc0010204, 0xc0010206, 0xc0010208, 0xc001020a,
};
static u32 PERF_CTR_MSR[6] = {
	0xc0010201, 0xc0010203, 0xc0010205, 0xc0010207, 0xc0010209, 0xc001020b,
};


// Zero out any PERF_CTL/PERF_CTR pairs that are disabled.
// If any of the counters are enabled, return an error.
int init_pmcs(void)
{
	u64 val;
	int err, i;
	for (i = 0; i < 6; i++)
	{
		err = rdmsrl_safe_on_cpu(TARGET_CPU, PERF_CTL_MSR[i], &val);
		if (err) {
			pr_err("lamina: invalid msr %08x (?)\n", PERF_CTL_MSR[i]);
			return -1;
		} else {
			if ((val & (1 << 22)) != 0) {
				pr_err("lamina: PERF_CTL[%d] is enabled\n", i);
				pr_err("lamina: all counters must be disabled\n");
				return -1;
			} else {
				wrmsrl_safe_on_cpu(TARGET_CPU, PERF_CTL_MSR[i], 0);
				wrmsrl_safe_on_cpu(TARGET_CPU, PERF_CTR_MSR[i], 0);
			}
		}	
	}
	return 0;
}


static __init int lamina_init(void)
{
	struct cpuinfo_x86 *info = &boot_cpu_data;
	if (!(info->x86_vendor == X86_VENDOR_AMD)) {
		pr_err("lamina: unsupported CPU\n");
		return -1;
	}

	// NOTE: Maybe its better to run this on TARGET_CPU ...
	if (!(native_read_cr4() & X86_CR4_PCE)) {
		pr_err("lamina: CR4.PCE is unset! - no RDPMC in user-space\n");
		return -1;
	}

	if (init_pmcs() != 0) {
		return -1;
	}
	if (misc_register(&lamina_dev) != 0) {
		pr_err("lamina: couldn't register device\n");
		return -1;
	}
	pr_info("lamina: loaded successfully\n");

	return 0;
}

static __exit void lamina_exit(void)
{
	misc_deregister(&lamina_dev);
	pr_info("lamina: unloaded module\n");
}

module_init(lamina_init);
module_exit(lamina_exit);
MODULE_LICENSE("GPL v2");
