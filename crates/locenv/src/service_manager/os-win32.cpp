#include "os.hpp"

#include <iostream>
#include <memory>
#include <ostream>
#include <sstream>
#include <stdexcept>

#include <errno.h>
#include <process.h>
#include <string.h>
#include <winsock2.h>

// windows.h required to included after the other headers otherwise it will cause redefinition error.
#include <windows.h>

static DWORD total;
static SOCKET sockets[WSA_MAXIMUM_WAIT_EVENTS];
static WSAEVENT events[WSA_MAXIMUM_WAIT_EVENTS];
static bool terminating;

static void shutdown(ULONG_PTR Parameter)
{
    terminating = true;
}

static std::unique_ptr<wchar_t[]> from_utf8(const char *utf8)
{
    // Get buffer size.
    auto bytes = static_cast<int>(strlen(utf8) + 1);
    auto required = MultiByteToWideChar(CP_UTF8, 0, utf8, bytes, nullptr, 0);

    if (!required) {
        auto code = GetLastError();
        std::stringstream message;

        message << "Cannot get a length for buffer to decode " << utf8 << " (" << code << ")";

        throw std::runtime_error(message.str());
    }

    // Decode.
    auto buffer = std::make_unique<wchar_t[]>(required);

    if (!MultiByteToWideChar(CP_UTF8, 0, utf8, bytes, buffer.get(), required)) {
        auto code = GetLastError();
        std::stringstream message;

        message << "Cannot decode " << utf8 << " (" << code << ")";

        throw std::runtime_error(message.str());
    }

    return buffer;
}

static LRESULT message_proc(HWND hWnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
    return 0;
}

extern "C" uint8_t enter_daemon(const char *log, unsigned (*daemon) (void *), void *context)
{
    // Create log file.
    auto name = from_utf8(log);
    auto handle = CreateFileW(name.get(), GENERIC_WRITE, FILE_SHARE_READ, nullptr, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, nullptr);

    if (handle == INVALID_HANDLE_VALUE) {
        auto c = GetLastError();
        std::stringstream m;

        m << "Cannot create " << log << " (" << c << ")";

        throw std::runtime_error(m.str());
    }

    // Set STDOUT.
    if (!SetStdHandle(STD_OUTPUT_HANDLE, handle)) {
        auto c = GetLastError();
        std::stringstream m;

        m << "Cannot use " << log << " as a standard output device (" << c << ")";

        throw std::runtime_error(m.str());
    }

    // Set STDERR.
    if (!SetStdHandle(STD_ERROR_HANDLE, handle)) {
        auto c = GetLastError();
        std::stringstream m;

        m << "Cannot use " << log << " as a standard error device (" << c << ")";

        throw std::runtime_error(m.str());
    }

    // Create a message-only window to receive event from Windows.
    WNDCLASSEXW wc = {0};

    wc.cbSize = sizeof(wc);
    wc.lpfnWndProc = message_proc;
    wc.hInstance = GetModuleHandleW(nullptr);
    wc.lpszClassName = L"locenv-service-manager";

    auto atom = RegisterClassExW(&wc);

    if (!atom) {
        auto code = GetLastError();
        std::stringstream m;

        m << "Failed to register a window class (" << code << ")";

        throw std::runtime_error(m.str());
    }

    auto wnd = CreateWindowExW(0, (LPCWSTR)atom, nullptr, 0, 0, 0, 0, 0, HWND_MESSAGE, nullptr, GetModuleHandleW(nullptr), nullptr);

    if (!wnd) {
        auto code = GetLastError();
        std::stringstream m;

        m << "Failed to create a window (" << code << ")";

        throw std::runtime_error(m.str());
    }

    // Start daemon in a separated thread due to Windows required the main thread to be message loop.
    auto runner = _beginthreadex(nullptr, 0, daemon, context, 0, nullptr);

    if (!runner) {
        auto code = errno;
        std::stringstream m;

        m << "Failed to create a thread to run the daemon (" << code << ")";

        throw std::runtime_error(m.str());
    }

    // Process events until WM_QUIT.
    MSG msg;
    BOOL res;

    while ((res = GetMessage(&msg, nullptr, 0, 0)) != 0) {
        if (res == -1) {
            auto code = GetLastError();
            std::stringstream m;

            m << "Failed to get a Windows message (" << code << ")";

            throw std::runtime_error(m.str());
        } else {
            TranslateMessage(&msg);
            DispatchMessage(&msg);
        }
    }

    // Stop daemon.
    if (!QueueUserAPC(shutdown, runner, 0)) {
        auto code = GetLastError();
        std::stringstream m;

        m << "Failed to stop daemon (" << code << ")";

        throw std::runtime_error(m.str());
    }

    if (WaitForSingleObject(runner, INFINITE) != WAIT_OBJECT_0) {
        throw std::runtime_error("Failed to wait for daemon");
    }

    // Get daemon exit code.
    DWORD status;

    if (!GetExitCodeThread(runner, &status)) {
        auto code = GetLastError();
        std::stringstream m;

        m << "Failed to get daemon status (" << code << ")";

        throw std::runtime_error(m.str());
    }

    CloseHandle(runner);

    return static_case<uint8_t>(status);
}

extern "C" void register_for_accept(SOCKET socket)
{
    if (total == WSA_MAXIMUM_WAIT_EVENTS) {
        throw std::runtime_error("The amount of connection has been exceed the limit");
    }

    // Create event handle.
    auto event = WSACreateEvent();

    if (event == WSA_INVALID_EVENT) {
        auto c = WSAGetLastError();
        std::stringstream m;

        m << "Failed to create event handle (" << c << ")";

        throw std::runtime_error(m.str());
    }

    // Listen for the event.
    if (WSAEventSelect(socket, event, FD_ACCEPT) == SOCKET_ERROR) {
        auto c = WSAGetLastError();
        std::stringstream m;

        m << "Failed to listen for connection (" << c << ")";

        throw std::runtime_error(m.str());
    }

    sockets[total] = socket;
    events[total] = event;
    total++;
}

extern "C" uint8_t dispatch_events(void (*handler) (SOCKET))
{
    if (!total) {
        return NO_EVENT_SOURCES;
    }

    for (;;) {
        auto result = WSAWaitForMultipleEvents(total, events, FALSE, WSA_INFINITE, TRUE);

        if (result == WSA_WAIT_FAILED) {
            auto c = WSAGetLastError();
            std::wcerr << L"Failed to wait for network events (" << c << L")" << std::endl;
            return WAIT_EVENTS_FAILED;
        } else if (result == WSA_WAIT_IO_COMPLETION) {
            if (terminating) {
                return DISPATCHER_TERMINATED;
            }

            continue;
        }

        for (auto i = result - WSA_WAIT_EVENT_0; i < total; i++) {
            // Reset event.
            if (WSAEventSelect(sockets[i], events[i], 0) == SOCKET_ERROR) {
                auto c = WSAGetLastError();
                std::wcerr << L"Failed to reset event notification (" << c << L")" << std::endl;
                return RESET_NOTIFICATION_FAILED;
            }

            WSACloseEvent(events[i]);

            handler(sockets[i]);
        }

        total = result - WSA_WAIT_EVENT_0;

        return SUCCESS;
    }
}
