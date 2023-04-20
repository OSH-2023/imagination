## Raspberry Pi

##### 什么是树莓派

树莓派是一款ARM架构的32位或64位机器（高版本型号都是64位）。它属于Cotex-A架构，因此具有很高的性能，具有MMU，可以运行linux等操作系统。

虽然树莓派就是一块很小的板子，但是其性能的优越使其可以当作一个迷你的个人PC用，许多PC上的接口比如USB、以太网、HDMI都能在树莓派上找到。树莓派可以做到大部分PC能做到的事情，比如将其当作一台服务器或者配合键鼠当作迷你笔记本电脑用；当然对于一些更简单的单片机都能完成的任务也可以用它来完成，树莓派同样提供一些MCU上常见的GPIO。

前些年由于树莓派的价格十分亲民，只需要几百元就能买到一台迷你的电脑，因此许多人买来做一些应用开发和学习，构建了其强大的生态环境。许多大学比赛里面树莓派经常会作为图像处理上位机的首选。但是近年来官方似乎放弃了其低价高性能的定位，价格越来越高，因此许多人正在积极寻找替代品，比如国产的orange pi。

目前树莓派已有多种型号，根据对硬件需求的不同和成本预算，可以自由地选择。

<img src="1.jpg" width="600">

##### 树莓派和QEMU

作为一款热门的ARM机器，QEMU已支持多种型号的树莓派，如下所示

|       型号       |       CPU        |  RAM   |
| :--------------: | :--------------: | :----: |
| raspi0和raspi1ap | ARM1176JZF-S单核 | 512MiB |
|     raspi2b      |  Cortex-A7四核   |  1GiB  |
|     raspi3ap     |  Cortex-A53四核  | 512MiB |
|     raspi3b      |  Cortex-A53四核  |  1GiB  |

只有树莓派2及以下型号是32位的，如果想要模拟64位的树莓派3，则不能用命令qemu-system-arm，需要改用qemu-system-aarch64，和STM32相类似，模拟树莓派只需要简短的命令，例如

```bash
$ qemu-system-arm -machine type=raspi2 -m 1024 -kernel vmlinux -initrd initramfs
$ qemu-system-aarch64 -machine type=raspi3 -m 1024 -kernel vmlinux -initrd initramfs
```

以上两个命令分别模拟了树莓派2和3，并指定内核为VMlinux及初始内存盘。

__为什么选择树莓派__

本次实验可能需要为FreeRTOS添加MMU相关功能，由于FreeRTOS本身对ARM架构支持是最好的，因此希望能选择ARM架构的机器进行测试，同时，由于64位的树莓派支持MMU，并且树莓派拥有较好的生态，网络上可以找到许多开发者分享的资料，因此树莓派成立本次项目的首选之一。

##### 参考资料

[What Is a Raspberry Pi? Here's What You Need to Know](https://www.makeuseof.com/what-is-raspberry-pi/)

[raspbian - How to emulate Raspberry Pi in QEMU?](https://raspberrypi.stackexchange.com/questions/117234/how-to-emulate-raspberry-pi-in-qemu)

[树莓派 wiki](https://zh.wikipedia.org/wiki/树莓派)

[Raspberry Pi boards (raspi0, raspi1ap, raspi2b, raspi3ap, raspi3b) — QEMU documentation](https://www.qemu.org/docs/master/system/arm/raspi.html)
