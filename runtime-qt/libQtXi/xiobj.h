#ifndef Xi_OBJ_H
#define Xi_OBJ_H

#include <new>
#include <stdint.h>

#include "libxi.h"

struct Xivtable {
};

struct Xiobj {
    Xiobj(): initialized(true)
    {}

    Xivtable* vptr;

    // This field is set to false when we are merely allocated
    // via GC, such as when an Xi subclass allocates our memory;
    // and true when we've actually called a constructor.
    bool initialized;
};

template<typename T> T* gcNew() {
    T* t = reinterpret_cast<T*>(_xi_alloc(sizeof(T)));
    new (t) T;
    registerFinalizer(t, T::invokeDtor);
    return t;
}

// Like gcNew, but doesn't deal with constructors or destructors
template<typename T> T* gcCalloc() {
    return reinterpret_cast<T*>(_xi_alloc(sizeof(T)));
}

inline int toXi(int v) {
    return v;
}

inline bool toXi(bool v) {
    return v;
}

inline int fromXi(int v) {
    return v;
}

inline bool fromXi(bool v) {
    return v;
}

inline void* fromXi(void* v) {
    return v;
}

enum ClassOp {
    CallCtor, // also responsible for calling createImpl
    CallDtor
};

struct DummyPeer
{};

#endif

// kate: indent-width 4; replace-tabs on; tab-width 4; space-indent on; hl c++;
