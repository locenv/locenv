#ifndef LOCENV_OS_HPP_INCLUDED
#define LOCENV_OS_HPP_INCLUDED

#ifdef _MSC_VER
#include <stdint.h>
#else
#include <inttypes.h>
#endif

extern "C" {
    extern const uint8_t SUCCESS;
    extern const uint8_t WAIT_CLIENT_FAILED;
    extern const uint8_t SELECT_FAILED;
    extern const uint8_t RESET_NOTIFICATION_FAILED;
    extern const uint8_t WAIT_EVENTS_FAILED;
    extern const uint8_t NO_EVENT_SOURCES;
    extern const uint8_t DISPATCHER_TERMINATED;
}

#endif // LOCENV_OS_HPP_INCLUDED
