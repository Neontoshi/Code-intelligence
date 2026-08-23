// tests/fixtures/adversarial/java/spring_controller.java

package com.example.controller;

import org.springframework.web.bind.annotation.*;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;

// ⚠️ This looks dead but is a Spring Controller
@RestController
@RequestMapping("/api/users")
public class UserController {

    @Autowired
    private UserService userService;

    // ⚠️ Looks dead but is a GET endpoint
    @GetMapping
    public List<User> getUsers() {
        return userService.findAll();
    }

    // ⚠️ Looks dead but is a GET endpoint with path variable
    @GetMapping("/{id}")
    public User getUser(@PathVariable Long id) {
        return userService.findById(id);
    }

    // ⚠️ Looks dead but is a POST endpoint
    @PostMapping
    public User createUser(@RequestBody User user) {
        return userService.save(user);
    }

    // ⚠️ Looks dead but is a PUT endpoint
    @PutMapping("/{id}")
    public User updateUser(@PathVariable Long id, @RequestBody User user) {
        return userService.update(id, user);
    }

    // ⚠️ Looks dead but is a DELETE endpoint
    @DeleteMapping("/{id}")
    public void deleteUser(@PathVariable Long id) {
        userService.delete(id);
    }

    // ⚠️ Looks dead but is a Service class used by the controller
    @Service
    public static class UserService {
        public List<User> findAll() { return new ArrayList<>(); }
        public User findById(Long id) { return new User(); }
        public User save(User user) { return user; }
        public User update(Long id, User user) { return user; }
        public void delete(Long id) { }
    }

    // Internal helper - should be considered dead
    private void internalHelper() {
        // This is actually dead
    }
}

class User {
    private Long id;
    private String name;
    // getters and setters...
}
