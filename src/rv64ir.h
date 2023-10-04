#include <stdint.h>

typedef struct CpuState {
    uint64_t regs[32];
    uint64_t pc;
    uint64_t csr[4096];
} CpuState;

void rv64ir_init(char* file_path, char* disk_path);
struct CpuState* rv64ir_cycle();
