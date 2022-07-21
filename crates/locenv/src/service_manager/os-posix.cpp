#include "os.hpp"

#include <iostream>
#include <ostream>
#include <sstream>
#include <stdexcept>

#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <string.h>
#include <sys/select.h>
#include <unistd.h>

static bool terminating;

static void handle_signal(int)
{
    terminating = true;
}

extern "C" void redirect_console_output(const char *file)
{
    // Create a file.
    auto fd = open(file, O_CREAT | O_WRONLY | O_TRUNC | O_CLOEXEC, S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);

    if (fd < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Cannot open " << file << ": " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Duplicate to STDOUT.
    if (dup2(fd, STDOUT_FILENO) < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Failed to set stdout to " << file << ": " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Duplicate it to STDERR.
    if (dup2(fd, STDERR_FILENO) < 0) {
        auto c = errno;
        std::stringstream m;

        m << "Failed to set stderr to " << file << ": " << strerror(c);

        throw std::runtime_error(m.str());
    }

    // Close the original FD.
    close(fd);
}

extern "C" uint8_t event_loop(int server, event_loop_handler_t handler, void *context)
{
    // Block all signals.
    sigset_t mask;

    sigfillset(&mask);

    if (sigprocmask(SIG_SETMASK, &mask, nullptr) < 0) {
        auto c = errno;
        std::cout << "Failed to block signals: " << strerror(c) << std::endl;
        return BLOCK_SIGNALS_FAILED;
    }

    // Set SIGTERM handler.
    struct sigaction act;

    memset(&act, 0, sizeof(act));

    act.sa_handler = handle_signal;

    if (sigaction(SIGTERM, &act, nullptr) < 0) {
        auto c = errno;
        std::cout << "Failed to install SIGTERM handler: " << strerror(c) << std::endl;
        return SIGACTION_FAILED;
    }

    // Enter event loop.
    int max = server + 1;
    fd_set descriptors;

    FD_ZERO(&descriptors);
    FD_SET(server, &descriptors);

    sigdelset(&mask, SIGTERM);

    for (;;) {
        // Wait for events.
        fd_set ready;
        int remaining;

        memcpy(&ready, &descriptors, sizeof(fd_set));

        if ((remaining = pselect(max, &ready, nullptr, nullptr, nullptr, &mask)) < 0) {
            auto c = errno;

            if (c == EINTR) {
                if (terminating) {
                    return SUCCESS;
                }

                continue;
            }

            std::cout << "Failed to wait for events: " << strerror(c) << std::endl;
            return SELECT_FAILED;
        }

        // Process event.
        for (int fd = 0; remaining && fd < max; fd++) {
            uint8_t res = 0;

            if (!FD_ISSET(fd, &ready)) {
                continue;
            }

            if (fd == server) {
                res = handler(context, LOCENV_CLIENT_CONNECT, reinterpret_cast<const void *>(fd));
            }

            if (res) {
                return res;
            }

            remaining--;
        }
    }
}
