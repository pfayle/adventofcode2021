<?php
use PHPUnit\Framework\TestCase;

class SubTest extends TestCase {
    private $sub;
    protected function setUp(): void {
        $this->sub = new Sub();
    }
    public function testInitialPosition(): void {
        $this->assertEquals([0,0],$this->sub->getPosition());
    }
    public function testForward(): void {
        $this->sub->doCommand("forward 5");
        $this->assertEquals([5,0],$this->sub->getPosition());
    }
    public function testDown(): void {
        $this->sub->doCommand("down 3");
        $this->assertEquals([0,3],$this->sub->getPosition());
    }
    public function testUp(): void {
        $this->sub->doCommand("up 1");
        $this->assertEquals([0,0],$this->sub->getPosition());
        $this->sub->doCommand("down 3");
        $this->sub->doCommand("up 2");
        $this->assertEquals([0,1],$this->sub->getPosition());
    }
    public function testProduct(): void {
        $this->sub->doCommand("forward 3");
        $this->sub->doCommand("down 4");
        $this->assertEquals(12, $this->sub->multiply());
    }
}