<?php
// tests/fixtures/adversarial/php/laravel_controller.php

namespace App\Http\Controllers;

class UserController
{
    // Web route target called dynamically via container / router
    public function show($id)
    {
        return $this->formatUserData($id);
    }

    // Dynamic reflection or callback target
    public function dynamicProcess($payload)
    {
        return call_user_func("strtoupper", $payload);
    }

    private function formatUserData($id)
    {
        return ["id" => $id];
    }

    // Dead function
    private function deadPhpFunction()
    {
        return null;
    }
}
