#include <sstream>
#include <stdexcept>

#include <errno.h>
#include <fcntl.h>
#include <string.h>
#include <unistd.h>

extern "C" int log_stderr(const char *path)
{
    // Create a file.
    auto fd = open(path, O_CREAT | O_WRONLY | O_TRUNC | O_CLOEXEC, S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);

    if (fd < 0) {
        auto code = errno;
        std::stringstream reason;

        reason << "Cannot open " << path << ": " << strerror(code);

        throw std::runtime_error(reason.str());
    }

    // Duplicate it to STDERR.
    if (dup2(fd, STDERR_FILENO) < 0) {
        auto code = errno;
        std::stringstream reason;

        reason << "Failed to set stderr to " << path << ": " << strerror(code);

        throw std::runtime_error(reason.str());
    }

    // Close the original FD.
    close(fd);

    return STDERR_FILENO;
}
