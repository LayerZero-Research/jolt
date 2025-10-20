// Minimal libc stubs for Jolt ZKVM
// This provides the bare minimum symbols that libc crate expects

#include <stdint.h>
#include <stddef.h>

// Basic type definitions that libc expects
typedef long time_t;
typedef int clockid_t;
typedef long suseconds_t;
typedef long ssize_t;

// iovec structure
struct iovec {
    void* iov_base;
    size_t iov_len;
};

// Minimal ioctl support
typedef unsigned long ioctl_t;
#define _IOC_NRBITS     8
#define _IOC_TYPEBITS   8
#define _IOC_SIZEBITS   14
#define _IOC_DIRBITS    2

#define _IOC_NRSHIFT    0
#define _IOC_TYPESHIFT  (_IOC_NRSHIFT+_IOC_NRBITS)
#define _IOC_SIZESHIFT  (_IOC_TYPESHIFT+_IOC_TYPEBITS)
#define _IOC_DIRSHIFT   (_IOC_SIZESHIFT+_IOC_SIZEBITS)

#define _IOC_NONE       0U
#define _IOC_WRITE      1U
#define _IOC_READ       2U

#define _IOC(dir,type,nr,size) \
        (((dir)  << _IOC_DIRSHIFT) | \
         ((type) << _IOC_TYPESHIFT) | \
         ((nr)   << _IOC_NRSHIFT) | \
         ((size) << _IOC_SIZESHIFT))

#define _IOR(type,nr,size)      _IOC(_IOC_READ,(type),(nr),sizeof(size))
#define _IOW(type,nr,size)      _IOC(_IOC_WRITE,(type),(nr),sizeof(size))
#define _IOWR(type,nr,size)     _IOC(_IOC_READ|_IOC_WRITE,(type),(nr),sizeof(size))

// Minimal errno support
extern int errno;
int errno = 0;

// Memory functions (basic stubs)
void* malloc(size_t size) { return (void*)0; }
void free(void* ptr) {}
void* calloc(size_t nmemb, size_t size) { return (void*)0; }
void* realloc(void* ptr, size_t size) { return (void*)0; }

// String functions (basic stubs - you may need to implement these properly)
void* memcpy(void* dest, const void* src, size_t n) {
    char* d = (char*)dest;
    const char* s = (const char*)src;
    for (size_t i = 0; i < n; i++) {
        d[i] = s[i];
    }
    return dest;
}

void* memset(void* s, int c, size_t n) {
    char* p = (char*)s;
    for (size_t i = 0; i < n; i++) {
        p[i] = c;
    }
    return s;
}

int memcmp(const void* s1, const void* s2, size_t n) {
    const unsigned char* p1 = (const unsigned char*)s1;
    const unsigned char* p2 = (const unsigned char*)s2;
    for (size_t i = 0; i < n; i++) {
        if (p1[i] < p2[i]) return -1;
        if (p1[i] > p2[i]) return 1;
    }
    return 0;
}

// System calls (all return error for ZKVM)
int open(const char* pathname, int flags, ...) { errno = 2; return -1; } // ENOENT
int close(int fd) { errno = 9; return -1; } // EBADF
ssize_t read(int fd, void* buf, size_t count) { errno = 9; return -1; } // EBADF
ssize_t write(int fd, const void* buf, size_t count) { errno = 9; return -1; } // EBADF

// Threading stubs (ZKVM is single-threaded)
int pthread_mutex_init(void* mutex, void* attr) { return 0; }
int pthread_mutex_lock(void* mutex) { return 0; }
int pthread_mutex_unlock(void* mutex) { return 0; }
int pthread_mutex_destroy(void* mutex) { return 0; }

// Time stubs
time_t time(time_t* tloc) { 
    if (tloc) *tloc = 0;
    return 0; 
}

int gettimeofday(void* tv, void* tz) { return 0; }

// Process stubs
int getpid(void) { return 1; }
void exit(int status) { 
    // In ZKVM, we should halt/abort
    while(1) {} 
}

void abort(void) {
    while(1) {}
}

// Signal stubs
int kill(int pid, int sig) { return 0; }
void* signal(int signum, void* handler) { return (void*)0; }

// Additional std library functions
int posix_memalign(void** memptr, size_t alignment, size_t size) {
    // Simple implementation - just use malloc for now
    *memptr = malloc(size);
    return (*memptr == NULL) ? -1 : 0;
}

ssize_t writev(int fd, const struct iovec* iov, int iovcnt) {
    // Stub implementation
    errno = 9; // EBADF
    return -1;
}

int* __errno_location(void) {
    return &errno;
}

long syscall(long number, ...) {
    // Stub implementation for syscalls
    errno = 38; // ENOSYS
    return -1;
}

// pthread stubs for thread-local storage
int pthread_key_create(unsigned int* key, void (*destructor)(void*)) {
    // Simple implementation - just return a unique key
    static unsigned int next_key = 1;
    *key = next_key++;
    return 0;
}

int pthread_key_delete(unsigned int key) {
    // No-op for ZKVM
    return 0;
}

void* pthread_getspecific(unsigned int key) {
    // Return NULL for all keys in single-threaded environment
    return NULL;
}

int pthread_setspecific(unsigned int key, const void* value) {
    // No-op for ZKVM
    return 0;
}