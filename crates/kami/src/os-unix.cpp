#include <errno.h>
#include <signal.h>
#include <string.h>

#include <sys/select.h>

static int max_fd = -1;
static fd_set readfds;

struct dispatch_handlers {
    int (*interrupted) (void *);
    void (*ready) (int, void *);
};

extern "C" int kami_pselect_init()
{
    if (max_fd != -1) {
        return 1;
    }

    max_fd = 0;

    FD_ZERO(&readfds);

    return 0;
}

extern "C" void kami_pselect_watch_accept(int fd)
{
    if (fd >= max_fd) {
        max_fd = fd + 1;
    }

    FD_SET(fd, &readfds);
}

extern "C" int kami_pselect_dispatch(const int *signals, int signals_count, const dispatch_handlers *handlers, void *context)
{
    if (!max_fd) {
        return 1;
    }

    // Set up signal mask.
    sigset_t mask;

    sigfillset(&mask);

    for (int i = 0; i < signals_count; i++) {
        sigdelset(&mask, signals[i]);
    }

    // Wait for events.
    for (;;) {
        fd_set readfds;
        int remaining;

        memcpy(&readfds, &::readfds, sizeof(fd_set));

        if ((remaining = pselect(max_fd, &readfds, nullptr, nullptr, nullptr, &mask)) < 0) {
            auto c = errno;

            if (c == EINTR) {
                if (!handlers->interrupted(context)) {
                    return 2;
                }

                continue;
            }

            return -c;
        }

        // Invoke ready handler.
        for (int fd = 0; remaining && fd < max_fd; fd++) {
            if (FD_ISSET(fd, &readfds)) {
                FD_CLR(fd, &::readfds);
            } else {
                continue;
            }

            handlers->ready(fd, context);
            remaining--;
        }

        return 0;
    }
}
