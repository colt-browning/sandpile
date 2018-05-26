extern crate sandpile;

use sandpile::{
	Sandpile,
	FiniteGrid,
	ToroidalGrid,
	png,
};

#[derive(Debug)]
enum GridType {
	Finite,
	Toroidal,
}

fn main() {
	let mut args = std::env::args().skip(1);
	let grid_type = match args.next() {
		Some(ref s) if s == "finite" => GridType::Finite,
		Some(ref s) if s == "torus"  => GridType::Toroidal,
		_ => {
			println!("Please specify grid type ('finite' or 'torus') as the 1st command line argument.");
			println!("Example of a correct call (with cargo, use 'cargo run --release' instead of 'sandpile'):");
			println!("sandpile finite 60x50 id ascii+png out/id.png");
			return;
		}
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
		Some(x) => x,
		None => {
			println!("Please specify grid size (as '100' or '200x100') as the 2nd command line argument.");
			return;
		}
	};
	match args.next() {
		Some(ref s) if s == "id" => (),
		_ => {
			println!("Please specify target (currenty only 'id' is supported) as the 3rd command line argument.");
			return;
		}
	};
	let (out_ascii, out_png) = match args.next() {
		Some(ref s) if s == "ascii" => (true, false),
		Some(ref s) if s == "png" => (false, true),
		Some(ref s) if s == "ascii+png" => (true, true),
		_ => {
			println!("Please specify output format ('ascii', 'png', or 'ascii+png') as the 4th command line argument.");
			return;
		}
	};
	let filename = if out_png {
		match args.next() {
			Some(s) => s,
			None => {
				println!("Please specify name for output png file as the 5th command line argument.");
				return;
			}
		}
	} else { String::new() };
	match grid_type {
		GridType::Finite => {
			let a = FiniteGrid::neutral(x, y);
			if out_ascii {
				print!("{}", a);
			}
			if out_png {
				let g = a.to_graph();
				if let Err(e) = png(&g, &filename) {
					println!("Can't write to file {}. {}", filename, e);
				}
			}
		},
		GridType::Toroidal => {
			let a = ToroidalGrid::neutral(x, y);
			if out_ascii {
				print!("{}", a);
			}
			if out_png {
				let g = a.to_graph();
				if let Err(e) = png(&g, &filename) {
					println!("Can't write to file {}. {}", filename, e);
				}
			}
		}
	};
}
