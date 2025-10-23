#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"

#include <foxdbg.h>

#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>
#include <time.h>
#include <signal.h>
#include <math.h>
#include <unistd.h>
#include <stdatomic.h>
#include <sched.h>

#if defined(_MSC_VER)
    #include <windows.h>
    #define YIELD_CPU() Sleep(0)
    #define ATOMIC_READ_INT(ptr) InterlockedCompareExchange((volatile LONG *)(ptr), 0, 0)
    #define ATOMIC_WRITE_INT(ptr, val) InterlockedExchange((volatile LONG *)(ptr), (val))
#else
    #define YIELD_CPU() sched_yield()
    #define ATOMIC_READ_INT(ptr) atomic_load_explicit((ptr), memory_order_seq_cst)
    #define ATOMIC_WRITE_INT(ptr, val) atomic_store_explicit((ptr), (val), memory_order_seq_cst)
#endif

static volatile sig_atomic_t is_running = 1;

void signal_handler(int signal)
{
    printf("Signal %d received, shutting down...\n", signal);
    is_running = 0;
}

/* Cross-platform high-resolution timer */
static double get_time_seconds(void)
{
#if defined(_WIN32)
    LARGE_INTEGER freq, counter;
    QueryPerformanceFrequency(&freq);
    QueryPerformanceCounter(&counter);
    return (double)counter.QuadPart / (double)freq.QuadPart;
#else
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec / 1e9;
#endif
}

int main(int argc, char *argv[])
{
    foxdbg_init();

    foxdbg_add_channel("/sensors/banana",  FOXDBG_CHANNEL_TYPE_IMAGE, 30);   
    foxdbg_add_channel("/sensors/banana2", FOXDBG_CHANNEL_TYPE_IMAGE, 30);
    foxdbg_add_channel("/waves/sin",  FOXDBG_CHANNEL_TYPE_FLOAT, 30);
    foxdbg_add_channel("/waves/bool", FOXDBG_CHANNEL_TYPE_BOOLEAN, 30);
    foxdbg_add_channel("/waves/int",  FOXDBG_CHANNEL_TYPE_INTEGER, 30);

    // int rx_channel = foxdbg_add_rx_channel("/rx/system_state", FOXDBG_CHANNEL_TYPE_BOOLEAN);

    signal(SIGINT,  signal_handler);
    signal(SIGTERM, signal_handler);

    int width, height, channels;
    uint8_t* data = stbi_load("c_test/banana.png", &width, &height, &channels, 3);  
    channels = 3;

    foxdbg_image_info_t image_info = { width, height, channels };
    foxdbg_write_channel_info("/sensors/banana", &image_info, sizeof(image_info));

    int width2, height2, channels2;
    uint8_t* data2 = stbi_load("c_test/banana.png", &width2, &height2, &channels2, 3);
    channels2 = 3;

    foxdbg_image_info_t image_info2 = { width2, height2, channels2 };
    foxdbg_write_channel_info("/sensors/banana2", &image_info2, sizeof(image_info2));

    while (is_running)
    {
        foxdbg_write_channel("/sensors/banana",  data,  width * height * channels);
        foxdbg_write_channel("/sensors/banana2", data2, width2 * height2 * channels2);

        double t = get_time_seconds();
        float sin_value = sinf((float)(t * 2.0f * 3.14159f * 0.1f));
        foxdbg_write_channel("/waves/sin", &sin_value, sizeof(sin_value));

        bool is_true = (sin_value > 0.0f);
        foxdbg_write_channel("/waves/bool", &is_true, sizeof(bool));

        static int int_value = 0;
        int_value++;
        foxdbg_write_channel("/waves/int", &int_value, sizeof(int_value));

        YIELD_CPU();
    }

    foxdbg_shutdown();
    return 0;
}

