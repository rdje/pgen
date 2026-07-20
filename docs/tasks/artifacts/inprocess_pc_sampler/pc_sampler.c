/*
 * PGEN-RGX-0078-0181: no-root in-process sampled-PC recorder for macOS/arm64.
 *
 * Loaded with DYLD_INSERT_LIBRARIES into the immutable preserved probe.  The
 * signal handler performs only a lock-free index reservation and one fixed
 * buffer store.  It never allocates, locks, formats, or performs I/O.  Output
 * happens after ITIMER_PROF is disabled and SIGPROF is blocked at teardown.
 */

#define _XOPEN_SOURCE 700

#include <errno.h>
#include <limits.h>
#include <mach-o/dyld.h>
#include <mach-o/loader.h>
#include <signal.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/time.h>
#include <ucontext.h>
#include <unistd.h>

#if !defined(__APPLE__) || !defined(__aarch64__)
#error "pc_sampler.c requires macOS arm64"
#endif

#if ATOMIC_INT_LOCK_FREE != 2
#error "sample index must be lock-free for signal-handler use"
#endif

#define SAMPLE_CAPACITY (1U << 20)
#define DEFAULT_INTERVAL_US 250U

static uintptr_t sampled_pcs[SAMPLE_CAPACITY];
static _Atomic unsigned int total_seen;
static unsigned int interval_us;
static char output_path[PATH_MAX];
static struct sigaction previous_sigprof;
static struct rusage start_usage;
static int sampler_active;

static intptr_t main_executable_slide(void) {
    const uint32_t image_count = _dyld_image_count();
    for (uint32_t index = 0; index < image_count; ++index) {
        const struct mach_header *header = _dyld_get_image_header(index);
        if (header != NULL && header->filetype == MH_EXECUTE) {
            return _dyld_get_image_vmaddr_slide(index);
        }
    }
    return INTPTR_MIN;
}

static void sample_pc(int signal_number, siginfo_t *signal_info, void *raw_context) {
    (void)signal_number;
    (void)signal_info;
    ucontext_t *context = (ucontext_t *)raw_context;
    const unsigned int index =
        atomic_fetch_add_explicit(&total_seen, 1U, memory_order_relaxed);
    if (index < SAMPLE_CAPACITY) {
        sampled_pcs[index] = (uintptr_t)context->uc_mcontext->__ss.__pc;
    }
}

static int parse_interval(const char *text, unsigned int *result) {
    if (text == NULL || *text == '\0') {
        *result = DEFAULT_INTERVAL_US;
        return 0;
    }
    char *end = NULL;
    errno = 0;
    const unsigned long value = strtoul(text, &end, 10);
    if (errno != 0 || end == text || *end != '\0' || value == 0 ||
        value > 1000000UL) {
        return -1;
    }
    *result = (unsigned int)value;
    return 0;
}

static uint64_t timeval_us(struct timeval value) {
    return (uint64_t)value.tv_sec * 1000000U + (uint64_t)value.tv_usec;
}

__attribute__((constructor)) static void start_sampler(void) {
    const char *requested_output = getenv("PGEN_PC_SAMPLE_OUT");
    if (requested_output == NULL || *requested_output == '\0') {
        return;
    }
    if (strlen(requested_output) >= sizeof(output_path)) {
        fprintf(stderr, "pgen-pc-sampler: output path is too long\n");
        return;
    }
    memcpy(output_path, requested_output, strlen(requested_output) + 1U);

    if (parse_interval(getenv("PGEN_PC_SAMPLE_INTERVAL_US"), &interval_us) != 0) {
        fprintf(stderr, "pgen-pc-sampler: invalid interval\n");
        return;
    }
    if (getrusage(RUSAGE_SELF, &start_usage) != 0) {
        fprintf(stderr, "pgen-pc-sampler: initial getrusage failed: %s\n", strerror(errno));
        return;
    }

    struct sigaction action;
    memset(&action, 0, sizeof(action));
    action.sa_sigaction = sample_pc;
    action.sa_flags = SA_SIGINFO | SA_RESTART;
    sigemptyset(&action.sa_mask);
    sigaddset(&action.sa_mask, SIGPROF);
    if (sigaction(SIGPROF, &action, &previous_sigprof) != 0) {
        fprintf(stderr, "pgen-pc-sampler: sigaction failed: %s\n", strerror(errno));
        return;
    }

    struct itimerval timer;
    memset(&timer, 0, sizeof(timer));
    timer.it_interval.tv_sec = (time_t)(interval_us / 1000000U);
    timer.it_interval.tv_usec = (suseconds_t)(interval_us % 1000000U);
    timer.it_value = timer.it_interval;
    if (setitimer(ITIMER_PROF, &timer, NULL) != 0) {
        fprintf(stderr, "pgen-pc-sampler: setitimer failed: %s\n", strerror(errno));
        (void)sigaction(SIGPROF, &previous_sigprof, NULL);
        return;
    }
    sampler_active = 1;
}

__attribute__((destructor)) static void stop_sampler(void) {
    if (!sampler_active) {
        return;
    }

    sigset_t block_set;
    sigset_t previous_mask;
    sigemptyset(&block_set);
    sigaddset(&block_set, SIGPROF);
    (void)sigprocmask(SIG_BLOCK, &block_set, &previous_mask);

    struct itimerval disabled;
    memset(&disabled, 0, sizeof(disabled));
    (void)setitimer(ITIMER_PROF, &disabled, NULL);
    (void)sigaction(SIGPROF, &previous_sigprof, NULL);
    sampler_active = 0;

    const unsigned int seen =
        atomic_load_explicit(&total_seen, memory_order_relaxed);
    const unsigned int stored = seen < SAMPLE_CAPACITY ? seen : SAMPLE_CAPACITY;
    const unsigned int dropped = seen - stored;
    const intptr_t main_slide = main_executable_slide();
    if (main_slide == INTPTR_MIN) {
        fprintf(stderr, "pgen-pc-sampler: cannot locate main executable slide\n");
        (void)sigprocmask(SIG_SETMASK, &previous_mask, NULL);
        return;
    }
    struct rusage end_usage;
    if (getrusage(RUSAGE_SELF, &end_usage) != 0) {
        fprintf(stderr, "pgen-pc-sampler: final getrusage failed: %s\n", strerror(errno));
        (void)sigprocmask(SIG_SETMASK, &previous_mask, NULL);
        return;
    }
    const uint64_t start_cpu_us =
        timeval_us(start_usage.ru_utime) + timeval_us(start_usage.ru_stime);
    const uint64_t end_cpu_us =
        timeval_us(end_usage.ru_utime) + timeval_us(end_usage.ru_stime);
    const uint64_t cpu_elapsed_us = end_cpu_us - start_cpu_us;

    FILE *output = fopen(output_path, "w");
    if (output == NULL) {
        fprintf(stderr, "pgen-pc-sampler: cannot open output: %s\n", strerror(errno));
        (void)sigprocmask(SIG_SETMASK, &previous_mask, NULL);
        return;
    }
    fprintf(output, "format=pgen-pc-samples-v2\n");
    fprintf(output, "pid=%d\n", getpid());
    fprintf(output, "timer=ITIMER_PROF\n");
    fprintf(output, "interval_us=%u\n", interval_us);
    fprintf(output, "capacity=%u\n", SAMPLE_CAPACITY);
    fprintf(output, "total_seen=%u\n", seen);
    fprintf(output, "stored=%u\n", stored);
    fprintf(output, "dropped=%u\n", dropped);
    fprintf(output, "cpu_elapsed_us=%llu\n", (unsigned long long)cpu_elapsed_us);
    fprintf(output, "main_slide=0x%llx\n", (unsigned long long)main_slide);
    for (unsigned int index = 0; index < stored; ++index) {
        fprintf(output, "pc=0x%llx\n", (unsigned long long)sampled_pcs[index]);
    }
    if (fclose(output) != 0) {
        fprintf(stderr, "pgen-pc-sampler: output close failed: %s\n", strerror(errno));
    }
    (void)sigprocmask(SIG_SETMASK, &previous_mask, NULL);
}
