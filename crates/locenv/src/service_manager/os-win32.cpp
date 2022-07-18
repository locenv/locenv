#include <memory>
#include <sstream>
#include <stdexcept>

#include <string.h>
#include <windows.h>

static std::unique_ptr<wchar_t[]> from_utf8(const char *utf8)
{
    // Get buffer size.
    auto bytes = (int)strlen(utf8) + 1;
    auto required = MultiByteToWideChar(CP_UTF8, 0, utf8, bytes, nullptr, 0);

    if (!required) {
        auto code = GetLastError();
        std::stringstream message;

        message << "Cannot get a length for buffer to decode " << utf8 << " (" << code << ")";

        throw std::runtime_error(message.str());
    }

    // Decode.
    auto buffer = std::make_unique<wchar_t>(required);

    if (!MultiByteToWideChar(CP_UTF8, 0, utf8, bytes, buffer.get(), required)) {
        auto code = GetLastError();
        std::stringstream message;

        message << "Cannot decode " << utf8 << " (" << code << ")";

        throw std::runtime_error(message.str());
    }

    return buffer;
}

extern "C" void * log_stderr(const char *path)
{
    // Create file.
    auto name = from_utf8(path);
    auto file = CreateFileW(name.get(), GENERIC_WRITE, FILE_SHARE_READ, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);

    if (file == INVALID_HANDLE_VALUE) {
        auto code = GetLastError();
        std::stringstream message;

        message << "Cannot create " << path << " (" << code << ")";

        throw std::runtime_error(message.str());
    }

    // Set STDERR.
    if (!SetStdHandle(STD_ERROR_HANDLE, file)) {
        auto code = GetLastError();
        std::stringstream message;

        message << "Cannot use " << path << " as a standard error device (" << code << ")";

        throw std::runtime_error(message.str());
    }

    return file;
}
