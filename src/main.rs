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
	let mut config = match Config::new(&mut std::env::args()) {
		Ok(config) => config,
		Err(e) => {
			println!("{}", e);
			return
		}
	};
	let (x, y) = config.dimensions;
	let mut stack = Vec::new();
	while let Some(action) = config.actions.pop() {
		match action {
			Action::Id => stack.push(GridSandpile::neutral(config.grid_type, config.dimensions)),
			Action::Read => match || -> Result<GridSandpile, Box<dyn Error>> {
				let mut g = String::new();
				for _ in 0..y {
					io::stdin().read_line(&mut g)?;
				}
				let mut a = GridSandpile::from_string(config.grid_type, config.dimensions, g)?;
				a.topple();
				Ok(a)
			}() {
				Ok(x) => stack.push(x),
				Err(e) => {
					println!("{}", e);
					return
				}
			},
			Action::ReadList => match read_list(x, y) {
				Ok(grid) => {
					let mut a = GridSandpile::from_grid(config.grid_type, grid).unwrap();
					a.topple();
					stack.push(a);
				},
				Err(e) => {
					println!("{}", e);
					return
				}
			},
			Action::All(n) => {
				let mut a = GridSandpile::from_grid(config.grid_type, vec![vec![n; x]; y]).unwrap();
				if n >= 4 {
					a.topple();
				}
				stack.push(a)
			},
			Action::Add => {
				let mut a = stack.pop().unwrap();
				if let Err(e) = a.add(&stack.pop().unwrap()) {
					println!("{}", e);
					return
				}
				stack.push(a)
			},
			Action::Dup => {
				let a = stack.last().unwrap().clone();
				stack.push(a);
			},
		}
	}
	let a = stack.pop().unwrap();
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
	actions: Vec<Action>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Action {
	Id,
	Read,
	ReadList,
	All(u8),
	Add,
	Dup,
//	_Inverse,
}

impl Config {
	fn new(args: &mut std::iter::Iterator<Item = String>) -> Result<Config, String> {
		args.next();
		let grid_type = match args.next() {
			Some(ref s) if s == "finite" => GridType::Finite,
			Some(ref s) if s == "torus" || s == "toroidal"  => GridType::Toroidal,
			_ => return Err("\
Please specify grid type ('finite' or 'torus') as the 1st command line argument.
Example of a correct call (with cargo, use 'cargo run --release' instead of 'sandpile'):
sandpile finite 60x50 ascii+png id out/id.png".to_owned())
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
			None => return Err("Please specify grid size (as '100' or '200x100') as the 2nd command line argument.".to_owned())
		};
		let (out_ascii, out_png) = match args.next() {
			Some(ref s) if s == "ascii" => (true, false),
			None => (true, false),
			Some(ref s) if s == "png" => (false, true),
			Some(ref s) if s == "ascii+png" => (true, true),
			Some(s) => return Err(format!("Please specify output format. Expected 'ascii' (default), 'png', or 'ascii+png', got: {}", s))
		};
		let mut actions_expected = 1;
		let mut actions = Vec::new();
		while actions_expected > 0 {
			let arg = match args.next() {
				Some(s) => s,
				None => return Err(if actions.is_empty() {
					"Please specify target: 'id', 'read', 'read_list', 'all-N', 'dup', or 'add'."
				} else {
					"Target list terminated unexpectedly."
				}.to_owned())
			};
			let (action, incr) = match arg.as_str() {
				"id" => (Action::Id, 0),
				"read" => (Action::Read, 0),
				"read_list" => (Action::ReadList, 0),
				s if s.starts_with("all-") => match s[4..].parse::<u8>() {
					Ok(n) => (Action::All(n), 0),
					Err(_e) => return Err("In target 'all-N', N must be a 8-bit number.".to_owned()),
				},
	//			"_inverse" => Action::_Inverse,
				"add" => (Action::Add, 2),
				"dup" => (Action::Dup, 0),
				s => return Err(format!("Unknown target: {}", s))
			};
			actions.push(action);
			actions_expected += incr - 1;
		}
		if *actions.last().unwrap() == Action::Dup {
			return Err("'dup' duplicates the next target, so at the point it occurs at least 2 targets should be expected, and at least 1 more should follow.".to_owned());
		}
		let filename = if out_png {
			match args.next() {
				Some(s) => s,
				None => return Err("Please specify name for output png file as the final command line argument.".to_owned())
			}
		} else { String::new() };
		Ok(Config {
			grid_type,
			dimensions: (x, y),
			out_ascii,
			out_png: if out_png { Some(filename) } else { None },
			actions,
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
