<?php
use PHPUnit\Framework\TestCase;

class Sub2Test extends TestCase {
    private $sub;
    protected function setUp(): void {
        $this->sub = new Sub2();
    }
    public function testInitials(): void {
        $this->assertEquals([0,0],$this->sub->getPosition());
        $this->assertEquals(0,$this->sub->getAim());
    }
    public function testDown(): void {
        $this->sub->doCommand("down 3");
        $this->assertEquals(3,$this->sub->getAim());
    }
    public function testUp(): void {
        $this->sub->doCommand("up 2");
        $this->assertEquals(-2,$this->sub->getAim());
    }
    public function testForward(): void {
        $this->sub->doCommand("down 3");
        $this->sub->doCommand("forward 5");
        $this->assertEquals(5,$this->sub->getPosition()[0]);
        $this->assertEquals(15,$this->sub->getPosition()[1]);
        $this->sub->doCommand("up 5");
        $this->sub->doCommand("forward 4");
        $this->assertEquals(9,$this->sub->getPosition()[0]);
        $this->assertEquals(7,$this->sub->getPosition()[1]);
    }
}