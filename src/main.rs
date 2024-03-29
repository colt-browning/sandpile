use sandpile::{
	GridType,
	FiniteGridType,
	Neighbourhood,
	GridSandpile,
	FiniteGridSandpile,
	png,
};

use std::{
	io,
	fs,
	error::Error,
	convert::TryFrom,
};

fn main() {
	if let Err(e) = (|| {
		let config = Config::new(&mut std::env::args())?;
		run(config)
	})() {
		eprintln!("{}", e);
	}
}

fn run(mut config: Config) -> Result<(), Box<dyn Error>> {
	let (x, y) = config.dimensions;
	let mut stack = Vec::new();
	let time = std::time::SystemTime::now();
	while let Some(action) = config.actions.pop() {
		match action {
			Action::Id => stack.push(FiniteGridSandpile::neutral(config.grid_type.finite()?, config.neighbourhood, config.dimensions)),
			Action::Burn => stack.push(FiniteGridSandpile::burn(config.grid_type.finite()?, config.neighbourhood, config.dimensions)),
			Action::Read => {
				let mut g = String::new();
				for _ in 0..y {
					io::stdin().read_line(&mut g)?;
				}
				let a = GridSandpile::from_string(config.grid_type, config.neighbourhood, config.dimensions, g)?;
				stack.push(a)
			},
			Action::ReadList => {
				let grid = read_list(x, y)?;
				let a = GridSandpile::from_grid(config.grid_type, config.neighbourhood, grid)?;
				stack.push(a)
			},
			Action::All(n) => {
				let a = GridSandpile::from_grid(config.grid_type, config.neighbourhood, vec![vec![n; x]; y])?;
				stack.push(a)
			},
			Action::Inverse => {
				let a = stack.pop().unwrap();
				let g = FiniteGridSandpile::try_from(&a)?.inverse();
				stack.push(g)
			}
			Action::Add => {
				let mut a = stack.pop().unwrap();
				a.add(&stack.pop().unwrap())?;
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
		return Ok(())
	}
	if config.topplings {
		println!("Topplings: {}", a.last_topple());
	}
	if config.chips_count {
		println!("Total chips count: {}", a.chips_count());
	}
	if config.order {
		println!("Order: {}", FiniteGridSandpile::try_from(&a)?.order());
	}
	if config.time {
		match time.elapsed() {
			Ok(t) => println!("Total time taken: {}.{} s", t.as_secs(), t.subsec_millis()),
			Err(e) => eprintln!("{}", e),
		}
	}
	if config.out_ascii {
		print!("{}", a);
	}
	if let Some((mut filename, colors)) = config.out_png {
		let g = a.into_grid();
		while let Err(e) = png(&g, &filename, &colors) {
			eprintln!("Can't write to file {}. {}", filename, e);
			eprintln!("Please enter correct name for output file:");
			filename = String::new();
			io::stdin().read_line(&mut filename)?;
			filename = filename.trim().to_string();
		}
	}
	Ok(())
}

#[derive(Debug)]
struct Config {
	grid_type: GridType,
	neighbourhood: Neighbourhood,
	dimensions: (usize, usize),
	out_ascii: bool,
	out_png: Option<(String, Vec<[u8; 4]>)>,
	eq: bool,
	order: bool,
	topplings: bool,
	chips_count: bool,
	time: bool,
	actions: Vec<Action>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Action {
	Id,
	Burn,
	Read,
	ReadList,
	All(sandpile::Cell),
	Add,
	Dup,
	Inverse,
}

impl Config {
	fn new(args: &mut std::env::Args) -> Result<Config, String> {
		args.next();
		let grid_type_err = Err("\
Please specify grid type ('rectangle', 'torus', or 'infinite') as the 1st command line argument.
To use Moore neighbourhood (8 neighbours), type 'rectangle.moore' etc.
Example of a correct call (with cargo, use 'cargo run --release' instead of 'sandpile'):
sandpile rectangle 60x50 ascii+png id out/id.png".to_owned());
		let grid_type = match args.next() {
			Some(s) => s,
			None => return grid_type_err
		};
		let grid_type: Vec<_> = grid_type.split('.').collect();
		let (grid_type, neighbourhood) = (grid_type[0], match grid_type.len() {
			1 => Neighbourhood::VonNeumann,
			2 if grid_type[1] == "moore" => Neighbourhood::Moore,
			_ => return grid_type_err
		});
		let grid_type = match grid_type {
			"rectangle" | "rectangular" | "finite" => GridType::Finite(FiniteGridType::Rectangular),
			"infinite" => GridType::Infinite(0, 0),
			"torus" | "toroidal"  => GridType::Finite(FiniteGridType::Toroidal),
			_ => return grid_type_err
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
		let mut time = false;
		let mut topplings = false;
		let mut chips_count = false;
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
				actions = vec![Action::Add, Action::Burn, Action::Dup];
				group = true;
			} else {
				for out in s.split("+") {
					match out {
						"ascii" => out_ascii = true,
						"png" => out_png = true,
						"time" => time = true,
						"topplings" => topplings = true,
						"chips" | "grains" => chips_count = true,
						"order" => {group = true; order = true},
						_ => return Err(format!("\
Expected output format
either '+'-separated 'ascii', 'png', 'time', 'topplings', 'chips', and/or 'order'
or sole 'eq' or 'recurrent'.
Got: {}", s))
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
					"Please specify command: 'id', 'read', 'read_list', 'all-N', 'burn', 'inverse', 'dup', or 'add'."
				} else {
					"Command list terminated unexpectedly."
				}.to_owned())
			};
			let (action, incr) = match arg.as_str() {
				"id" => {group = true; (Action::Id, 0)},
				"burn" => {group = true; (Action::Burn, 0)},
				"read" => (Action::Read, 0),
				"read_list" => (Action::ReadList, 0),
				s if s.starts_with("all-") => match s[4..].parse::<sandpile::Cell>() {
					Ok(n) => (Action::All(n), 0),
					Err(_e) => return Err("In command 'all-N', N must be a 32-bit number.".to_owned()),
				},
				"inverse" => {group = true; (Action::Inverse, 1)},
				"add" => (Action::Add, 2),
				"dup" => (Action::Dup, 0),
				s => return Err(format!("Unknown command: {}", s))
			};
			actions.push(action);
			actions_expected += incr - 1;
		}
		if *actions.last().unwrap() == Action::Dup {
			return Err("'dup' duplicates the top sandpile on the stack, so at the point it occurs at least 2 commands should be expected, and at least 1 more command should follow.".to_owned());
		}
		let out_png = if out_png {
			let colors = get_colors()?;
			match args.next() {
				Some(filename) => Some((filename, colors)),
				None => return Err("Please specify name for output png file as the final command line argument.".to_owned())
			}
		} else { None };
		if grid_type.finite().is_err() && group {
			return Err("For the infinite grid, outputs 'order' and 'recurrent' and commands 'id', 'burn', and 'inverse' are impossible.".to_owned())
		}
		Ok(Config {
			grid_type,
			neighbourhood,
			dimensions: (x, y),
			out_ascii,
			out_png,
			eq,
			order,
			topplings,
			chips_count,
			time,
			actions,
		})
	}
}

fn read_list(x: usize, y: usize) -> Result<sandpile::Grid, Box<dyn Error>> {
	let mut g = String::new();
	while !g.ends_with(".") {
		io::stdin().read_line(&mut g)?;
		g = g.trim_end().to_string();
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

fn hex_to_int(s: &str) -> Option<[u8; 4]> {
	let ch: Vec<_> = s.chars().collect();
	if ch.len() < 6 {
		return None
	}
	let mut color = [255; 4];
	for i in 0..3 {
		color[2-i] = (ch[2*i].to_digit(16)? * 16 + ch[2*i+1].to_digit(16)?) as u8
	}
	Some(color)
}

fn get_colors() -> Result<Vec<[u8; 4]>, String> {
	let colors_file = match fs::read_to_string("colors") {
		Ok(s) => s,
		Err(e) => return Err(e.to_string())
	};
	let mut colors = Vec::new();
	for (n, line) in colors_file.lines().enumerate() {
		if let Some(color) = hex_to_int(line) {
			colors.push(color)
		} else if n >= 2 {
			break
		} else {
			return Err("\
Less than 2 valid lines (beginning with 6 hex digits) found
in the file 'colors', so sensible png output is impossible.".into())
		}
	}
	Ok(colors)
}
