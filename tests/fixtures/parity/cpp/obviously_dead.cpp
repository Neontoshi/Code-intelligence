#include <iostream>

void helperFunction() {
    std::cout << "This is never called" << std::endl;
}

int main() {
    std::cout << "Entry point" << std::endl;
    return 0;
}
