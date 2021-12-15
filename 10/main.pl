package Main;
use strict;
use warnings;
use Readonly;
our $VERSION = 1.0;
Readonly::Scalar my $LIB => [
    [ '(', ')', 3 ],
    [ '[', ']', 57 ],
    [ '{', '}', 1197 ],
    [ '<', '>', 25_137 ]
];
Readonly::Scalar my $MULTIPLIER => 5;

my $invalid_score      = 0;
my $autocorrect_scores = [];

sub type {
    my ($c) = @_;
    my $i = 0;
    foreach my $row ( @{$LIB} ) {
        if ( $c eq $row->[0] ) {
            return [ 'OPEN', $i ];
        }
        elsif ( $c eq $row->[1] ) {
            return [ 'CLOSE', $i ];
        }
        $i++;
    }
    return [ 'OTHER', 0 ];
}

sub process_line {
    my ($line) = @_;
    chomp $line;
    my @stack = ();
    foreach my $c ( split //sm, $line ) {
        my $t = type($c);
        if ( 'OPEN' eq $t->[0] ) {
            push @stack, $t->[1];
        }
        elsif ( 'CLOSE' eq $t->[0] ) {
            my $r = pop @stack;
            if ( $r eq $t->[1] ) {
                next;
            }
            else {
                invalid($t);
                return;    # corrupt line
            }
        }
    }
    if ( 0 == @stack ) {
        return    # complete line
    }
    my $autocorrect_score = 0;
    while ( @stack > 0 ) {
        my $r = pop @stack;
        $autocorrect_score = score( $autocorrect_score, $r );
    }
    push @{$autocorrect_scores}, $autocorrect_score;
    return;
}

sub invalid {
    my ($t) = @_;
    $invalid_score += $LIB->[ $t->[1] ]->[2];
    return;
}

sub score {
    my ( $autocorrect_score, $r ) = @_;
    return $MULTIPLIER * $autocorrect_score + ( $r + 1 );
}

while (<>) {
    process_line $_;
}

my $rv    = print "Error score: $invalid_score\n";
my $index = @{$autocorrect_scores} / 2;
$rv = print 'Middle incomplete score: '
  . ( sort { $a <=> $b } @{$autocorrect_scores} )[$index] . "\n";

1;
