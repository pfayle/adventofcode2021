<?php
class Sub2 {
    private $position = [0,0];
    private $aim = 0;

    public function doCommand($command) {
        [$verb, $quantity] = explode(" ", $command);
        if ("forward" == $verb) {
            $this->position[0] += $quantity;
            $this->position[1] += $quantity * $this->aim;
        } elseif("down" == $verb) {
            $this->aim += $quantity;
        } elseif("up" == $verb) {
            $this->aim -= $quantity;
        }
    }

    public function getPosition() {
        return $this->position;
    }

    public function multiply() {
        return $this->position[0] * $this->position[1];
    }

    public function getAim() {
        return $this->aim;
    }
}