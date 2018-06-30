extern crate sandpile;

use sandpile::{
	GridType,
	GridSandpile,
	png,
};

use std::{
	io,
	error::Error
};

fn main() {
	let config = match Config::new(&mut std::env::args()) {
		Ok(config) => config,
		Err(e) => {
			println!("{}", e);
			return
		}
	};
	let (x, y) = config.dimensions;
	let mut a = match config.action {
		Action::Id => GridSandpile::neutral(config.grid_type, config.dimensions),
		Action::Read => match || -> Result<GridSandpile, Box<dyn Error>> {
			let mut g = String::new();
			for _ in 0..y {
				io::stdin().read_line(&mut g)?;
			}
			Ok(GridSandpile::from_string(config.grid_type, config.dimensions, g)?)
		}() {
			Ok(x) => x,
			Err(e) => {
				println!("{}", e);
				return
			}
		},
		Action::ReadList => match read_list(x, y) {
			Ok(grid) => GridSandpile::from_grid(config.grid_type, grid).unwrap(),
			Err(e) => {
				println!("{}", e);
				return
			}
		},
		Action::All(n) => GridSandpile::from_grid(config.grid_type, vec![vec![n; x]; y]).unwrap(),
	};
	match config.action {
		Action::Read | Action::ReadList => a.topple(),
		Action::All(n) if n >= 4 => a.topple(),
		_ => 0
	};
	if config.out_ascii {
		print!("{}", a);
	}
	if let Some(mut filename) = config.out_png {
		let g = a.into_grid();
		while let Err(e) = png(&g, &filename) {
			println!("Can't write to file {}. {}", filename, e);
			println!("Please enter correct name for output file:");
			filename = String::new();
			if let Err(e) =
				io::stdin().read_line(&mut filename) {
				println!("{}", e);
				return
			};
			filename = filename.trim().to_string();
		}
	}
}

#[derive(Debug)]
struct Config {
	grid_type: GridType,
	dimensions: (usize, usize),
	out_ascii: bool,
	out_png: Option<String>,
	action: Action,
}

#[derive(Debug, PartialEq)]
enum Action {
	Id,
	Read,
	ReadList,
	All(u8),
//	_Inverse,
}

impl Config {
	fn new(args: &mut std::iter::Iterator<Item = String>) -> Result<Config, &str> {
		args.next();
		let grid_type = match args.next() {
			Some(ref s) if s == "finite" => GridType::Finite,
			Some(ref s) if s == "torus" || s == "toroidal"  => GridType::Toroidal,
			_ => return Err("\
Please specify grid type ('finite' or 'torus') as the 1st command line argument.
Example of a correct call (with cargo, use 'cargo run --release' instead of 'sandpile'):
sandpile finite 60x50 id ascii+png out/id.png")
		};
		let (x, y) = match || -> Option<_> {
			let s = match args.next() {
				Some(x) => x,
				None => return None
			};
			if let Ok(x) = s.parse::<usize>() {
				if x > 0 {
					return Some((x, x))
				}
			}
			let sx: Vec<_> = s.split("x").collect();
			if sx.len() != 2 {
				return None
			}
			if let (Ok(x), Ok(y)) = (sx[0].parse::<usize>(), sx[1].parse::<usize>()) {
				if x > 0 && y > 0 {
					return Some((x, y))
				}
			}
			None
		}() {
			Some(dim) => dim,
			None => return Err("Please specify grid size (as '100' or '200x100') as the 2nd command line argument.")
		};
		let action = match args.next() {
			Some(ref s) if s == "id" => Action::Id,
			Some(ref s) if s == "read" => Action::Read,
			Some(ref s) if s == "read_list" => Action::ReadList,
			Some(ref s) if s.starts_with("all-") => match s[4..].parse::<u8>() {
				Ok(n) => Action::All(n),
				Err(_e) => return Err("In target 'all-N', N must be a 8-bit number."),
			},
//			Some(ref s) if s == "_inverse" => Action::_Inverse,
			_ => return Err("Please specify target ('id', 'read', 'read_list', or 'all-N' where N is number) as the 3rd command line argument.")
		};
		let (out_ascii, out_png) = match args.next() {
			Some(ref s) if s == "ascii" => (true, false),
			Some(ref s) if s == "png" => (false, true),
			Some(ref s) if s == "ascii+png" => (true, true),
			_ => return Err("Please specify output format ('ascii', 'png', or 'ascii+png') as the 4th command line argument.")
		};
		let filename = if out_png {
			match args.next() {
				Some(s) => s,
				None => return Err("Please specify name for output png file as the 5th command line argument.")
			}
		} else { String::new() };
		Ok(Config {
			grid_type,
			dimensions: (x, y),
			out_ascii,
			out_png: if out_png { Some(filename) } else { None },
			action,
		})
	}
}

fn read_list(x: usize, y: usize) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
	let mut g = String::new();
	while !g.ends_with(".") {
		io::stdin().read_line(&mut g)?;
		g = g.trim_right().to_string();
	}
	let mut grid = vec![vec![0; x]; y];
	for s in g[..g.len()-1].split_terminator(',') {
		let ss: Vec<_> = s.split_whitespace().collect();
		if ss.len() == 0 {
			continue
		}
		if ss.len() != 2 {
			return Err(format!("Expected 2 coordinates, got {}: {}", ss.len(), s).into())
		}
		let (xc, yc): (usize, usize) = (ss[0].parse()?, ss[1].parse()?);
		if xc >= x || yc >= y {
			return Err(format!("Coordinates ({}, {}) out of bounds (0..{}, 0..{})", xc, yc, x, y).into())
		}
		grid[yc][xc] += 1;
	}
	Ok(grid)
}
