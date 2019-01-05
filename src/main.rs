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
			Action::Read => match || -> Result<_, Box<dyn Error>> {
				let mut g = String::new();
				for _ in 0..y {
					io::stdin().read_line(&mut g)?;
				}
				let a = GridSandpile::from_string(config.grid_type, config.dimensions, g)?;
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
					let a = GridSandpile::from_grid(config.grid_type, grid).unwrap();
					stack.push(a);
				},
				Err(e) => {
					println!("{}", e);
					return
				}
			},
			Action::All(n) => {
				let a = GridSandpile::from_grid(config.grid_type, vec![vec![n; x]; y]).unwrap();
				stack.push(a)
			},
			Action::Inverse => {
				let a = stack.pop().unwrap();
				let g = a.inverse();
				stack.push(g);
			}
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
	if config.eq {
		let a2 = stack.pop().unwrap();
		println!("{}", a == a2);
		return
	}
	if config.out_ascii {
		print!("{}", a);
	}
	if config.topplings {
		println!("Topplings: {}", a.last_topple());
	}
	if config.order {
		println!("Order: {}", a.order());
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
	eq: bool,
	order: bool,
	topplings: bool,
	actions: Vec<Action>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Action {
	Id,
	Read,
	ReadList,
	All(sandpile::Cell),
	Add,
	Dup,
	Inverse,
}

impl Config {
	fn new(args: &mut std::iter::Iterator<Item = String>) -> Result<Config, String> {
		args.next();
		let grid_type = match args.next() {
			Some(ref s) if s == "finite" => GridType::Finite,
			Some(ref s) if s == "infinite" => GridType::Infinite(0, 0),
			Some(ref s) if s == "torus" || s == "toroidal"  => GridType::Toroidal,
			_ => return Err("\
Please specify grid type ('finite', 'torus', or 'infinite') as the 1st command line argument.
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
		let mut group = false;
		let mut out_ascii = false;
		let mut out_png = false;
		let mut topplings = false;
		let mut order = false;
		let mut eq = false;
		let mut actions = Vec::new();
		let mut actions_expected = 1;
		if let Some(s) = args.next() {
			if s == "eq" {
				eq = true;
				actions_expected = 2;
			} else if s == "recurrent" {
				eq = true;
				actions = vec![Action::Add, Action::Id, Action::Dup];
				group = true;
			} else {
				for out in s.split("+") {
					match out {
						"ascii" => out_ascii = true,
						"png" => out_png = true,
						"topplings" => topplings = true,
						"order" => {group = true; order = true},
						_ => return Err(format!("\
Expected output format
either '+'-separated 'ascii', 'png', 'topplings', and/or 'order'
or sole 'eq' or 'recurrent'.
Got: {}", out))
					}
				}
			}
		} else {
			return Err("Please specify desired output (e.g., 'ascii') as the 3rd command line argument.".to_owned())
		};
		while actions_expected > 0 {
			let arg = match args.next() {
				Some(s) => s,
				None => return Err(if actions.is_empty() {
					"Please specify target: 'id', 'read', 'read_list', 'all-N', 'inverse', 'dup', or 'add'."
				} else {
					"Target list terminated unexpectedly."
				}.to_owned())
			};
			let (action, incr) = match arg.as_str() {
				"id" => {group = true; (Action::Id, 0)},
				"read" => (Action::Read, 0),
				"read_list" => (Action::ReadList, 0),
				s if s.starts_with("all-") => match s[4..].parse::<sandpile::Cell>() {
					Ok(n) => (Action::All(n), 0),
					Err(_e) => return Err("In target 'all-N', N must be a 32-bit number.".to_owned()),
				},
				"inverse" => {group = true; (Action::Inverse, 1)},
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
		if let GridType::Infinite(..) = grid_type {
			if group {
				return Err("For the infinite grid, outputs 'order' and 'recurrent' and targets 'id' and 'inverse' are impossible.".to_owned())
			}
		}
		Ok(Config {
			grid_type,
			dimensions: (x, y),
			out_ascii,
			out_png: if out_png { Some(filename) } else { None },
			eq,
			order,
			topplings,
			actions,
		})
	}
}

fn read_list(x: usize, y: usize) -> Result<sandpile::Grid, Box<dyn Error>> {
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
