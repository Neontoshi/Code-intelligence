// tests/fixtures/adversarial/cpp/virtual_member.cpp

#include <iostream>
#include <string>

class BaseHandler {
public:
    virtual ~BaseHandler() = default;
    virtual void handle(const std::string& msg) = 0;
};

// Polymorphic implementation: virtual method call site is decoupled
class ConcreteHandler : public BaseHandler {
public:
    void handle(const std::string& msg) override {
        std::cout << "Handling: " << msg << std::endl;
    }
};

// JNI / FFI export pattern: called from external native runtime
extern "C" void Java_com_app_Native_runNative(void* env, void* obj) {
    ConcreteHandler handler;
    handler.handle("init");
}

// Truly dead function
static void unused_cpp_static_helper() {
    // dead
}
