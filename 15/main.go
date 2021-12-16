package main

import (
	"bufio"
	"errors"
	"flag"
	"fmt"
	"log"
	"os"
)

type cell struct {
	row      int
	col      int
	risk     int
	distance int
	parent   *cell
	visited  bool
}

var cave [][]*cell

var times = flag.Int("times", 1, "number of times to extend the original grid")

func (c *cell) visit() {
	c.visited = true
	for _, c2 := range c.unvisited_neighbours() {
		c2.update_distance(c)
	}
}

func (c *cell) update_distance(p *cell) int {
	new_distance := p.distance + c.risk
	if c.distance == -1 || c.distance > new_distance {
		c.distance = new_distance
		c.parent = p
	}
	return c.distance
}

func next_unvisited_cell() (*cell, error) {
	var smallest *cell
	d := -1
	for _, row := range cave {
		for _, c := range row {
			if c.visited {
				continue
			}
			if c.distance > 0 && (d == -1 || c.distance < d) {
				d = c.distance
				smallest = c
			}
		}
	}
	if d == -1 {
		return nil, errors.New("No eligible cells remaining")
	}
	return smallest, nil
}

func (c *cell) unvisited_neighbours() []*cell {
	var ret []*cell
	c2, err := get_cell(c.col, c.row-1)
	if err == nil && !c2.visited {
		ret = append(ret, c2)
	}
	c2, err = get_cell(c.col, c.row+1)
	if err == nil && !c2.visited {
		ret = append(ret, c2)
	}
	c2, err = get_cell(c.col-1, c.row)
	if err == nil && !c2.visited {
		ret = append(ret, c2)
	}
	c2, err = get_cell(c.col+1, c.row)
	if err == nil && !c2.visited {
		ret = append(ret, c2)
	}
	return ret
}

func get_cell(c int, r int) (*cell, error) {
	if len(cave) <= r {
		return nil, errors.New("Outside cave")
	}
	if len(cave[0]) <= c {
		return nil, errors.New("Outside cave")
	}
	if r < 0 || c < 0 {
		return nil, errors.New("Outside cave")
	}
	return cave[r][c], nil
}

func (c *cell) last() bool {
	return c.row == len(cave)-1 && c.col == len(cave[0])-1
}

func main() {
	flag.Parse()
	reader := bufio.NewReader(os.Stdin)
	r := 0
	for {
		line, err := reader.ReadString('\n')
		if err != nil {
			break
		}
		var row []*cell
		c := 0
		for _, v := range line {
			if v == '\n' {
				continue
			}
			row = append(row, &cell{
				row:      r,
				col:      c,
				risk:     int(v - '0'),
				distance: -1,
				visited:  false,
			})
			c++
		}
		cave = append(cave, row)
		r++
	}

	cave = extrapolate(cave, *times)

	current := cave[0][0]
	current.distance = 0
	for {
		current.visit()

		if current.last() {
			break
		}

		var err error
		current, err = next_unvisited_cell()
		if err != nil {
			log.Fatal(err)
		}
	}
	fmt.Printf("Shortest route length: %d\n", cave[len(cave)-1][len(cave[0])-1].distance)
}

func extrapolate(cave [][]*cell, times int) [][]*cell {
	rows := len(cave)
	cols := len(cave[0])
	for i := 1; i < times; i++ {
		for ri, r := range cave {
			for ci := 0; ci < cols; ci++ {
				c := &cell{
					row:      ri,
					col:      ci + i*cols,
					risk:     (r[ci].risk+i-1)%9 + 1,
					distance: -1,
					visited:  false,
				}
				cave[ri] = append(cave[ri], c)
			}
		}
	}
	for i := 1; i < times; i++ {
		for ri := 0; ri < rows; ri++ {
			var r []*cell
			for ci, c := range cave[ri] {
				c2 := &cell{
					row:      ri + i*rows,
					col:      ci,
					risk:     (c.risk+i-1)%9 + 1,
					distance: -1,
					visited:  false,
				}
				r = append(r, c2)
			}
			cave = append(cave, r)
		}
	}
	return cave
}
