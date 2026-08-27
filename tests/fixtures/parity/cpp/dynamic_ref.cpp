#include <iostream>
#include <map>
#include <functional>

std::map<std::string, std::function<void()>> handlers;

void register_handler(const std::string& name, std::function<void()> fn) {
    handlers[name] = fn;
}

int main() {
    register_handler("test", []() { std::cout << "Handler" << std::endl; });
    handlers["test"]();
    return 0;
}
