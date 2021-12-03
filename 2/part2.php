<?php
require_once __DIR__ . "/src/Sub2.php";

$sub = new Sub2();
$contents = file_get_contents($argv[1]);
$lines = explode("\n", $contents);
foreach($lines as $line) {
    $sub->doCommand($line);
}
$coords = $sub->getPosition();
printf("Sub position: (%d,%d)\n", $coords[0], $coords[1]);
printf("Multiplication result: %d\n", $coords[0] * $coords[1]);
