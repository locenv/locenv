#include <string.h>

#include <winsock2.h>

static DWORD total = (DWORD)-1;
static SOCKET sockets[WSA_MAXIMUM_WAIT_EVENTS];
static WSAEVENT events[WSA_MAXIMUM_WAIT_EVENTS];

extern "C" int kami_winsock_event_init()
{
    if (total != (DWORD)-1) {
        return 1;
    }

    total = 0;

    return 0;
}

extern "C" int kami_winsock_event_watch_accept(SOCKET socket)
{
    if (total == WSA_MAXIMUM_WAIT_EVENTS) {
        return 1;
    }

    auto event = WSACreateEvent();

    if (event == WSA_INVALID_EVENT) {
        return -WSAGetLastError();
    }

    if (WSAEventSelect(socket, event, FD_ACCEPT) == SOCKET_ERROR) {
        return -WSAGetLastError();
    }

    sockets[total] = socket;
    events[total] = event;
    total++;

    return 0;
}

extern "C" int kami_winsock_event_watch_read(SOCKET socket)
{
    if (total == WSA_MAXIMUM_WAIT_EVENTS) {
        return 1;
    }

    auto event = WSACreateEvent();

    if (event == WSA_INVALID_EVENT) {
        return -WSAGetLastError();
    }

    if (WSAEventSelect(socket, event, FD_READ) == SOCKET_ERROR) {
        return -WSAGetLastError();
    }

    sockets[total] = socket;
    events[total] = event;
    total++;

    return 0;
}

extern "C" int kami_winsock_event_watch_write(SOCKET socket)
{
    if (total == WSA_MAXIMUM_WAIT_EVENTS) {
        return 1;
    }

    auto event = WSACreateEvent();

    if (event == WSA_INVALID_EVENT) {
        return -WSAGetLastError();
    }

    if (WSAEventSelect(socket, event, FD_WRITE) == SOCKET_ERROR) {
        return -WSAGetLastError();
    }

    sockets[total] = socket;
    events[total] = event;
    total++;

    return 0;
}

extern "C" int kami_winsock_event_watch_remove(SOCKET socket)
{
    for (DWORD i = 0; i < total; i++) {
        if (sockets[i] == socket) {
            if (WSAEventSelect(sockets[i], events[i], 0) == SOCKET_ERROR) {
                return -WSAGetLastError();
            }

            WSACloseEvent(events[i]);

            total--;
            memmove(&sockets[i], &sockets[i + 1], (total - i) * sizeof(SOCKET));
            memmove(&events[i], &events[i + 1], (total - i) * sizeof(WSAEVENT));

            return 0;
        }
    }

    return 1;
}

extern "C" int kami_winsock_event_dispatch(void (*handler) (SOCKET, void *), void *context)
{
    if (!total) {
        return 1;
    }

    auto result = WSAWaitForMultipleEvents(total, events, FALSE, WSA_INFINITE, TRUE);

    if (result == WSA_WAIT_FAILED) {
        return -WSAGetLastError();
    } else if (result == WSA_WAIT_IO_COMPLETION) {
        return 2;
    }

    for (auto i = result - WSA_WAIT_EVENT_0; i < total; i++) {
        if (WSAEventSelect(sockets[i], events[i], 0) == SOCKET_ERROR) {
            return -WSAGetLastError();
        }

        WSACloseEvent(events[i]);

        handler(sockets[i], context);
    }

    total = result - WSA_WAIT_EVENT_0;

    return 0;
}
