.PHONY: all clean
all:
	@echo "Building kernel module ..."
	$(MAKE) -C kernel/
clean:
	$(MAKE) -C kernel/ clean
