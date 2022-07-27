#ifndef LOCENV_OS_HPP_INCLUDED
#define LOCENV_OS_HPP_INCLUDED

#ifdef _MSC_VER
#include <stdint.h>
#else
#include <inttypes.h>
#endif

extern "C" {
    extern const uint8_t SUCCESS;
}

#endif // LOCENV_OS_HPP_INCLUDED
