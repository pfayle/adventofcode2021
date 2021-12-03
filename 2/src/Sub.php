<?php
class Sub {
    private $position = [0,0];

    public function doCommand($command) {
        [$verb, $quantity] = explode(" ", $command);
        if ("forward" == $verb) {
            $this->position[0] += $quantity;
        } elseif("down" == $verb) {
            $this->position[1] += $quantity;
        } elseif("up" == $verb) {
            $this->position[1] -= min($quantity, $this->position[1]);
        }
    }

    public function getPosition() {
        return $this->position;
    }

    public function multiply() {
        return $this->position[0] * $this->position[1];
    }
}