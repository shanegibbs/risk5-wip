# Risk5

## U53-MC

* [u54-mc](https://www.sifive.com/cores/u54-mc)
* [U54-MC-RVCoreIP.pdf](https://static.dev.sifive.com/U54-MC-RVCoreIP.pdf)

## Device Tree

* [Device Tree Usage](https://elinux.org/Device_Tree_Usage)

## INTC

* [riscv%2Ccpu-intc.txt](https://github.com/torvalds/linux/blob/master/Documentation/devicetree/bindings/interrupt-controller/riscv%2Ccpu-intc.txt)

* Was [drivers/irqchip/irq-riscv-intc.c](https://github.com/shanegibbs/riscv-linux/blob/fe92d7905c6ea0ebeabeb725b8040754ede7c220/drivers/irqchip/irq-riscv-intc.c), is now [arch/riscv/kernel/irq.c](https://github.com/torvalds/linux/blob/master/arch/riscv/kernel/irq.c)

## PLIC

* [sifive%2Cplic-1.0.0.txt](https://github.com/torvalds/linux/blob/master/Documentation/devicetree/bindings/interrupt-controller/sifive%2Cplic-1.0.0.txt)

* Was [drivers/irqchip/irq-riscv-plic.c](https://github.com/shanegibbs/riscv-linux/blob/fe92d7905c6ea0ebeabeb725b8040754ede7c220/drivers/irqchip/irq-riscv-plic.c), is now: [drivers/irqchip/irq-sifive-plic.c](
https://github.com/torvalds/linux/blob/e5f6d9afa3415104e402cd69288bb03f7165eeba/drivers/irqchip/irq-sifive-plic.c)
