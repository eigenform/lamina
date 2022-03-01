# SPDX-License-Identifier: GPL-2.0

obj-m 			:= lamina.o
lamina-y 		:= fops.o main.o
KMOD_DIR 		:= /lib/modules/$(shell uname -r)/build
#EXTRA_CFLAGS 	:= -g -I$(CURDIR)/../include
EXTRA_CFLAGS 	:= -g

all:
	$(MAKE) -C $(KMOD_DIR) M=$(CURDIR) CFLAGS="$(CFLAGS)" \
		EXTRA_CFLAGS="$(EXTRA_CFLAGS)"
dis:
	objdump -Mintel -d lamina.ko
clean:
	$(MAKE) -C $(KMOD_DIR) M=$(CURDIR) clean
