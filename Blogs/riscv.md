
## ABI

ABI 即 Application binary interface, 是硬件层面的两段程序间调用的接口, 由处理器体系结构规定. 其内容包括:

- 数据类型(处理器使用的基本类型)的大小、字段/位域表示、对齐方式等.
- 调用约定(控制函数如何传送和接受返回值), 包括寄存器使用约定、参数传递顺序、栈的增长方向等(编译器配合实现).
- 如何进行系统调用.
- 其他

### 通用寄存器及其使用约定

**riscv共有32个通用整数寄存器:**

| 寄存器别名         | 寄存器            | 描述        | 保存责任         | 说明                                                                                                      |
| ------------- | -------------- | --------- | ------------ | ------------------------------------------------------------------------------------------------------- |
| zero          | x0             | 零寄存器      | 无需保存         | 值恒为0, 一般用于清零操作.                                                                                         |
| ra            | x1             | 返回地址      | 调用者保存        | 使用跳转指令会自动赋值返回地址给`ra`寄存器, `ret`指令会自动跳转回`ra`指向的地址; 调用者必须主动保存其原返回地址.                                       |
| sp            | x2             | 栈指针       | 被调用者保存       | 指向当前栈顶, 函数调用时由被调用者保存和恢复.                                                                                |
| gp            | x3             | 全局指针      | 无需显式保存       | 指向全局数据区域基地址, 通常在进程启动时由操作系统/启动代码初始化, 进程上下文切换时由操作系统保存和恢复, 无需在函数调用时保存.                                     |
| tp            | x4             | 线程指针      | 无需显式保存       | 指向线程局部存储(TLS)区域基地址, 通常在线程创建时由操作系统/线程库初始化, 线程上下文切换时由操作系统保存和恢复, 无需在函数调用时保存.                               |
| t0-t2, t3-t6  | x5-x7, x28-x31 | 临时寄存器     | 无保存责任(调用者保存) | 调用者可直接使用, 无需保存, 分为`t0-t2`,`t3-t6`两组.                                                                    |
| s0-s1, s2-s11 | x8-x9, x18-x27 | 保存寄存器     | 被调用者保存       | 被调用者需在函数开始时保存、在函数返回前恢复所使用到的寄存器值, 分为`s0-s1`, `s2-s11`两组.                                                 |
| a0-a7         | x10-x17        | 参数/返回值寄存器 | 调用者保存        | 其中`a0-a7`可用于函数调用传参(前8个参数), `a0-a1`可用于保存返回值. 调用者使用`a0-a7`传参前需主动保存其中的值, 函数返回后(且读取返回值后)恢复使用到的`a0-a7`寄存器的值. |

- **对于寄存器的保存职责归属问题, 若未规定被调用者保存, 则对于任何之后需要使用的通用寄存器数据, 调用者在函数调用前自然都要保存.**
- **RISC-V 中 fp 寄存器就是 bp 寄存器, 实际上是复用 s0(x8) 寄存器.**

> _此外还有32个通用浮点寄存器(f0~f31); 32个通用向量寄存器(v0~v31)(用于SIMD向量计算)_

### 一般栈帧结构

![一般栈帧结构](http://localhost:6001/RustOS/assets/framestruct.png) (fp不应该指向ra起始处而应该指向prev fp起始处?)

### 访存与地址对齐

riscv支持灵活的访存粒度(如1/2/4/8字节), 具体有指令决定. 例如:

- LB/LBU 指令以 1 字节粒度加载数据.
- LD 指令以 8 字节粒度加载数据(64 位架构).

当CPU使用多个字节为单位访问物理地址时要注意**地址对齐**问题. 对于 RISC-V 处理器而言, load/store 指令进行数据访存时, 数据在内存中的地址应该对齐. 如果访存 32 位数据, 内存地址应当按 32 位(4字节)对齐 如果数据的地址没有对齐, 执行访存操作将产生异常! 即使在一些架构中允许非对齐访问, 非对齐访问也会导致性能问题(例如需要**多次访存**).

### RISC-V 指令

> riscv 基本指令长度是 4 字节, 支持压缩指令扩展(RVC, RISCV Compressed Instructions)的两字节长度指令, 必须分别以四字节和两字节对齐.

- ​`mhartid`​: Machine Hardware Thread ID, 是一个 CSR(Control and Status Register), 为只读寄存器, 需通过 `csrr`​ 指令读取.
    
- ​`csrr`​: Control and Status Register Read.
    
- ​`auipc`​: Add Upper Immediate to PC, 例如 `auipc t0,0x10`​ 功能为 `t0 = PC + (0x10 << 12)`​, 低位累加立即数一般通过 `addi`​ 指令.
    
- ​`addi`​: Add Immediate, 格式为 `addi 目标寄存器,源寄存器,立即数`​.
    
- ​`jal`​: Jump and Link, 使用指定寄存器值 + 立即数偏移跳转, 立即数范围为 `+-1MB`​(指令提供一位符号位和二十位数值位). 且将返回地址(该跳转指令的下一条指令地址)自动保存到寄存器 `ra(x1)`​. _注意跳转地址都会向低地址以两字节对齐, 即最低位强制置为 0._
    
- ​`j`​: Jump, 是 `jal`​ 的伪指令, `j offset`​ 等价于 `jal x0, offset`​.
    
- ​`jalr`​: Jump and Link Register, 寄存器跳转, 格式：`jalr rd, rs1, imm`（或简化形式如 `jalr rd, imm(rs1)`）。
    
    - rd：目标寄存器，用于保存返回地址（通常是x1，即ra寄存器）。
    - rs1：基寄存器，包含跳转目标的基地址。
    - imm：立即数偏移（-2048到2047的12位有符号数）。
- ​`jr`​: Jump Register, 寄存器跳转且不保存返回地址, 是 `jalr`​ 的伪指令; `jr t0`​ 等价于 `jalr x0, 0(t0)`​, 目标寄存器为 `x0`​(零寄存器, 丢弃返回地址), 立即数为 0 表示无偏移.
    
- ​`unimp`​: Unimplemented Instruction, 一般说明此处为数据 0x00, 执行该指令会触发非法指令异常.
    
- ​`lb、lh、lw、ld`​: Load Byte、Load Halfword、Load Word、Load DoubleWord, 分别用于从内存加载 8 位、16 位、32 位、64 位数据到寄存器, 操作数不可以使用立即数.
    
- ​`sb、sh、sw、sd`​: Store Doubleword、Store Halfword、Store Word、Store DoubleWord, 分别用于将 8 位、16 位、32 位、64 位数据存储到内存, 操作数不可以使用立即数.
    
- ​`ret`​: Return, 是 `jalr x0, 0(ra)` ​的伪指令.
    
- `csrrs`: Control and Status Register Read and Set, 指令格式和语义如下:  
    `csrrs rd, csr, rs # rd ← CSR[csr]; CSR[csr] ← CSR[csr] | rs`  
    第一步 将目标 CSR 的当前值写入通用寄存器 rd  
    第二步 将该 CSR 的值与 rs 的值按位或, 结果写回该 CSR  
    **注意: 即使第一步中rd是第二步中的rs, rs也会使用该指令执行前的值**  
    **具有原子性**
    
- `csrrc`: Control and Status Register Read and Clear, 指令格式和语义同理如下:  
    `csrrc rd, csr, rs # rd ← CSR[csr]; CSR[csr] ← CSR[csr] & ~rs` **具有原子性**
    
- `csrrw`: Control and Status Register Read and Write, 指令格式和语义同理如下:  
    `csrrw rd, csr, rs # rd ← CSR[csr]; CSR[csr] ← rs` **具有原子性**
    

### 特权级切换

**相关CSR寄存器:**

- sstatus(Supervisor Status Register):  
    控制监督模式下的全局状态, 类似 x86 的 `EFLAGS`.
    
    关键字段:
    
    |位域|名称|功能|
    |---|---|---|
    |​`SPP` (Supervisor Previous Privilege)​|前特权级|记录异常发生前的特权级(0=用户态，1=监督态). `0`对应 U-mode, `1`对应 S-mode|
    |​`SPIE` (Supervisor Previous Interrupt Enable)​|中断使能|异常发生前的中断使能状态(恢复时用).|
    |​`SIE` (Supervisor Interrupt Enable)​|中断开关|控制监督模式下的中断使能(1=允许中断，0=禁止).|
    |​`SUM` (Supervisor User Memory access)​|内存权限|允许监督态访问用户态内存(用于系统调用).|
    
- sepc(Supervisor Exception Program Counter):  
    保存发生异常或中断时的程序计数器(PC)​, 即触发异常的指令地址. 之后通过 sret 指令返回到 sepc 指向的地址.
    
- scause(Supervisor Cause Register): 记录异常或中断的原因(最高位表示是中断还是异常), 类似 x86 的 `CR2`.
    
    字段:
    
    |位|含义|常见值(部分示例)|
    |---|---|---|
    |​`63`​|中断/异常标志|1=中断，0=异常。|
    |​`62:0`​|原因码(Exception Code)|​`2=非法指令`​,`5=加载地址错误`​,`8=用户态ecall`​,`9=监督态ecall`​|
    
    原因码一般用于在异常处理程序中根据其分发处理逻辑.
    
- stval(Supervisor Trap Value Register): 保存异常相关的附加信息, 例如存储访问的具体不合法地址、存储不合法指令的具体编码, 类似 x86 的 `CR2`.
    
- stvec(Supervisor Trap Vector base address Register): 设置异常(Trap)处理程序的入口地址(同时处理异常和中断).
    
    字段:
    
    | 63 | 62:2 | 1:0 |
    |---|---|---|
    |Reserved|BASE (Trap Vector Base)|MODE|
    
    当 Trap 触发时, 如果 `MODE = 0` 则无论 `cause` 为何值, `PC` 都自动跳转到地址 `BASE`; 如果 `MODE = 1` 则 `PC` 自动跳转到地址 `BASE + 4 × cause`.
    

**当 CPU 执行完一条指令准备从 U-mode Trap to S-mode 时, RISCV 硬件自动完成以下处理:**

1. sstatus 的 `SPP` 字段修改为 CPU 当前特权级
2. sepc 修改为 Trap 指令下一条指令地址
3. scause/stval 分别修改为此次 Trap 原因及相关附加信息
4. CPU 跳转到 stvec 设置的 Trap 处理程序入口地址, 同时将当前特权级设置为 S-mode, 然后执行 Trap 处理程序