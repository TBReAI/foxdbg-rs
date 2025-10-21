#include <stdio.h>
#include "foxdbg.h"
#include<unistd.h>

int main() {
    printf("Calling foxdbg_init() from C...\n");
    foxdbg_init();

    printf("Calling foxdbg_add_channel() from C...\n");
    foxdbg_add_channel("Test", 0, 0);

    for (int i=0; i<100; i++) {
      foxdbg_write_channel("Test", 0, 1);
    }

    foxdbg_shutdown();
    return 0;
  }
