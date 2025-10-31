#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"

#include <foxdbg.h>

#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>
#include <time.h>
#include <signal.h>
#include <math.h>

#if defined(_MSC_VER)
    #include <windows.h>
    #define YIELD_CPU() Sleep(0)
    #define sleep(seconds) Sleep((seconds) * 1000)
    #define ATOMIC_READ_INT(ptr) InterlockedCompareExchange((volatile LONG *)(ptr), 0, 0)
    #define ATOMIC_WRITE_INT(ptr, val) InterlockedExchange((volatile LONG *)(ptr), (val))
#else
    #include <unistd.h>
    #include <sched.h>
    #include <stdatomic.h>
    #include <stdlib.h>
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
    foxdbg_add_channel("/pointclouds/test", FOXDBG_CHANNEL_TYPE_POINTCLOUD, 30);

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

    const int num_points = 10000;
    foxdbg_vector4_t* pointcloud = (foxdbg_vector4_t*)malloc(num_points * sizeof(foxdbg_vector4_t));

    foxdbg_add_channel("/cubes/test", FOXDBG_CHANNEL_TYPE_CUBES, 30);
    const int num_cubes = 1;
    foxdbg_cube_t* cubes = (foxdbg_cube_t*)malloc(num_cubes * sizeof(foxdbg_cube_t));

    foxdbg_add_channel("/lines/test", FOXDBG_CHANNEL_TYPE_LINES, 30);
    const int num_lines = 2;
    foxdbg_line_t* lines = (foxdbg_line_t*)malloc(num_lines * sizeof(foxdbg_line_t));

    foxdbg_add_channel("/poses/test", FOXDBG_CHANNEL_TYPE_POSE, 30);
    foxdbg_pose_t* pose = (foxdbg_pose_t*)malloc(sizeof(foxdbg_pose_t));

    foxdbg_add_channel("/tf", FOXDBG_CHANNEL_TYPE_TRANSFORM, 30);
    foxdbg_transform_t* transform = (foxdbg_transform_t*)malloc(sizeof(foxdbg_transform_t));

    foxdbg_add_channel("/location/test", FOXDBG_CHANNEL_TYPE_LOCATION, 1);
    foxdbg_location_t* location = (foxdbg_location_t*)malloc(sizeof(foxdbg_location_t));

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

        for (int i = 0; i < num_points; i++) {
            float time_offset = (float)t * 2.0f;
            pointcloud[i].x = sinf(i * 0.1f + time_offset) * 5.0f;
            pointcloud[i].y = cosf(i * 0.1f + time_offset) * 5.0f;
            pointcloud[i].z = i * 0.1f - 5.0f;
            pointcloud[i].w = 255.f;
        }
        foxdbg_write_channel("/pointclouds/test", pointcloud, num_points * sizeof(foxdbg_vector4_t));

        cubes[0].position.x = sinf((float)t * 0.5f) * 5.0f;
        cubes[0].position.y = cosf((float)t * 0.5f) * 5.0f;
        cubes[0].position.z = 0.0f;

        cubes[0].size.x = 1.0f;
        cubes[0].size.y = 1.0f;
        cubes[0].size.z = 1.0f;

        cubes[0].orientation.x = 0.0f;
        cubes[0].orientation.y = 0.0f;
        cubes[0].orientation.z = (float)t;

        cubes[0].color.r = 1.0f;
        cubes[0].color.g = 0.0f;
        cubes[0].color.b = 0.0f;
        cubes[0].color.a = 1.0f;

        foxdbg_write_channel("/cubes/test", cubes, num_cubes * sizeof(foxdbg_cube_t));

        // Create a rotating line
        lines[0].start.x = 0;
        lines[0].start.y = 0;
        lines[0].start.z = 0;
        lines[0].end.x = sinf((float)t) * 5.0f;
        lines[0].end.y = cosf((float)t) * 5.0f;
        lines[0].end.z = 2.0f;
        lines[0].thickness = 0.1f;
        lines[0].color.r = 0.0f;
        lines[0].color.g = 1.0f;
        lines[0].color.b = 0.0f;
        lines[0].color.a = 1.0f;

        // Create a static line
        lines[1].start.x = 0;
        lines[1].start.y = 0;
        lines[1].start.z = 0;
        lines[1].end.x = 5;
        lines[1].end.y = 0;
        lines[1].end.z = 0;
        lines[1].thickness = 0.1f;
        lines[1].color.r = 0.0f;
        lines[1].color.g = 0.0f;
        lines[1].color.b = 1.0f;
        lines[1].color.a = 1.0f;

        foxdbg_write_channel("/lines/test", lines, num_lines * sizeof(foxdbg_line_t));

        // Create a moving and rotating pose
        pose->position.x = sinf((float)t * 0.5f) * 2.0f;
        pose->position.y = cosf((float)t * 0.5f) * 2.0f;
        pose->position.z = 1.0f;
        pose->orientation.x = 0.0f;
        pose->orientation.y = 0.0f;
        pose->orientation.z = (float)t;
        pose->color.r = 1.0f;
        pose->color.g = 1.0f;
        pose->color.b = 0.0f;
        pose->color.a = 1.0f;

        foxdbg_write_channel("/poses/test", pose, sizeof(foxdbg_pose_t));

        // Create a transform
        transform->id = "child_frame";
        transform->parent_id = "world";
        transform->position.x = 1.0f;
        transform->position.y = 2.0f;
        transform->position.z = 0.0f;
        transform->orientation.x = 0.0f;
        transform->orientation.y = 0.0f;
        transform->orientation.z = (float)t; // Rotating

        foxdbg_write_channel("/tf", transform, sizeof(foxdbg_transform_t));

        // Create a location
        location->timestamp_sec = (uint32_t)t;
        location->timestamp_nsec = (uint32_t)((t - floor(t)) * 1e9);
        location->latitude = 51.5074; // London
        location->longitude = -0.1278 + (double)sinf((float)t * 0.1f) * 0.1; // Moving back and forth
        location->altitude = 11.0;

        foxdbg_write_channel("/location/test", location, sizeof(foxdbg_location_t));

        YIELD_CPU();
    }

    free(pointcloud);
    free(cubes);
    free(lines);
    free(pose);
    free(transform);
    free(location);
    foxdbg_shutdown();
    return 0;
}

