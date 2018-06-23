extern crate sandpile;

use sandpile::{
	GridType,
	GridSandpile,
	png,
};

fn main() {
	let config = match Config::new(&mut std::env::args()) {
		Ok(config) => config,
		Err(e) => {
			println!("{}", e);
			return
		}
	};
	let a = GridSandpile::neutral(config.grid_type, config.dimensions);
	if config.out_ascii {
		print!("{}", a);
	}
	if let Some(mut filename) = config.out_png {
		let g = a.into_grid();
		while let Err(e) = png(&g, &filename) {
			println!("Can't write to file {}. {}", filename, e);
			println!("Please enter correct name for output file:");
			filename = String::new();
			std::io::stdin().read_line(&mut filename).unwrap();
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
		match args.next() {
			Some(ref s) if s == "id" => (),
			_ => return Err("Please specify target (currenty only 'id' is supported) as the 3rd command line argument.")
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
		})
	}
}
