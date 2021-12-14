require 'pp'

STDOUT.sync = true

class Cell
    attr_reader :value
    def initialize(row, col, val)
        @row = row
        @column = col
        @value = val
        @marked = false
        @row.cells.push self
        @column.cells.push self
    end

    def mark(n)
        if n == @value then
            @marked = true
            self
        end
    end

    def won?
        @row.won? || @column.won?
    end

    def marked?
        @marked
    end
end

class Row
    attr_accessor :cells
    attr_reader :id
    def initialize(type, id)
        @type = type
        @id = id
        @cells = []
        @marked = false
    end

    def mark(n)
        @cells.find do |c|
            c.mark n
        end
    end

    def cell(n)
        @cells.find { |c| c.value == n }
    end

    def won?
        @cells.reduce(true) { |bool, c| bool && c.marked? }
    end

    def print
        puts "#{@type} #{@id}"
        puts @cells.map { |c| c.marked? }.join(' ')
        @cells.each { |c| puts "#{c.marked?} #{c.value}" }
    end
end

class Board
    attr_reader :winning_number
    def initialize(stream)
        @won = false
        @rows = []
        @columns = []
        @score = 0
        (1..5).each do |i|
            col = Row.new "column", i
            @columns.push col
        end
        (1..5).each do |i|
            row = Row.new "row", i
            @rows.push row
        end
        @rows.each do |row|
            @columns.each do |col|
                val = stream.shift
                cell = Cell.new(row, col, val)
            end
        end
    end

    def mark(n)
        return false unless row = @rows.find do |r| # stop after first success
            r.mark(n)
        end
        return false unless cell = row.cell(n)
        win n if @score == 0 && cell.won?
    end

    def win(n)
        @winning_number = n
        @score = calculate_score
    end

    def won?
        @winning_number
    end

    def calculate_score
        @rows.sum do |r|
            r.cells.filter { |c| c.marked? == false }
            .sum { |c| c.value }
        end
    end

    def print
        puts won? ? "Won with number #{@winning_number}!" : "Has not won"
        puts "Score: #{@score}"
        puts "Final score: #{@winning_number * @score}" if won?
        puts
    end
end

# input processing: read first line into one stream, split by commas
# read rest of file into second stream split by whitespace including newlines
streams = ARGF.readlines.map(&:chomp)

calls = streams.shift.split(',').map(&:to_i)
board_nums = streams.join(' ').split(' ').map(&:to_i)

boards = []
while board_nums.size >= 25 do
    boards.push Board.new(board_nums)
end

winners = []

calls.each do |n|
    boards.each do |b|
        b.mark n
        winners.push(b) if !winners.include?(b) && b.won?
    end
end

winners.first.print
winners.last.print
