terraform {
  required_providers {
    null = {
      source = "hashicorp/null"
      version = "3.1.0"
    }
    local = {
      source = "hashicorp/local"
      version = "2.1.0"
    }
  }
}

provider "null" {
}

provider "local" {
}

variable "inputfile" {
    type = string
}

variable "part" {
    type = number
    default = 2
}

data "local_file" "input" {
    filename = var.inputfile
}

locals {
    lines = compact(split("\n", data.local_file.input.content))
    parsed_lines = [for i, v in local.lines : regex("(\\d+),(\\d+) -> (\\d+),(\\d+)", v)]
    # ignore each if not h/v (part 1 only)
    filtered_lines = [for i, v in local.parsed_lines : v if v[0] == v[2] || v[1] == v[3]]
    points = concat([for i, v in (var.part == 1 ? local.filtered_lines : local.parsed_lines) : [for j in range(max(abs(v[2]-v[0]), abs(v[3]-v[1])) + 1) : [v[0]+j*(v[2]==v[0] ? 0 : (v[2]-v[0])/abs(v[2]-v[0])), v[1]+j*(v[3]==v[1] ? 0 : (v[3]-v[1])/abs(v[3]-v[1]))] ] ])
    ordered_points = concat(local.points...)
    points_map = {for i, v in local.ordered_points : format("%s,%s", tostring(v[0]), tostring(v[1])) => 0... }
    points_count = {for k, v in local.points_map : k => length(v)}
    points_filtration = {for k, v in local.points_count : k => v if v > 1}
}

output "lines" {
    value = local.lines
}

output "filtered_points" {
    value = local.points_filtration
}

output "answer" {
    value = length(local.points_filtration)
}
