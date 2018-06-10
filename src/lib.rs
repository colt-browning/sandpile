extern crate repng;

use std::{
	collections::HashSet,
	io,
	fs::File,
	fmt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridType {
	Finite,
	Toroidal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSandpile {
	grid_type: GridType,
	grid: Vec<Vec<u8>>,
}

impl fmt::Display for GridSandpile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", fmt_grid(&self.grid))
	}
}

impl GridSandpile {
	pub fn from_grid(grid_type: GridType, grid: Vec<Vec<u8>>) -> Result<GridSandpile, &'static str> {
		if grid.is_empty() {
			return Err("Zero-size grid");
		}
		let l = grid[0].len();
		if l == 0 {
			return Err("Empty first row");
		}
		for row in &grid {
			if row.len() != l {
				return Err("Rows of unequal lengths");
			}
		}
		let mut sandpile = GridSandpile {
			grid_type,
			grid,
		};
		if grid_type == GridType::Toroidal {
			sandpile.grid[0][0] = 0;
		}
		Ok(sandpile)
	}

	pub fn add(&mut self, p: &GridSandpile) -> Result<(), &str> {
		if p.grid_type != self.grid_type
		|| p.grid.len() != self.grid.len()
		|| p.grid[0].len() != self.grid[0].len() {
			return Err(ADD_ERR_MSG);
		}
		for i in 0..self.grid.len() {
			for j in 0..self.grid[0].len() {
				self.grid[i][j] += p.grid[i][j];
			}
		}
		self.topple();
		Ok(())
	}
	
	pub fn neutral(grid_type: GridType, (x, y): (usize, usize)) -> GridSandpile {
	// Proposition 6.36 of http://people.reed.edu/~davidp/divisors_and_sandpiles/
		let mut sandpile = GridSandpile::from_grid(grid_type, vec![vec![6; x]; y]).unwrap(); // TODO: ?
		sandpile.topple();
		for mut row in &mut sandpile.grid {
			for mut el in row {
				*el = 6 - *el;
			}
		}
		if grid_type == GridType::Toroidal {
			sandpile.grid[0][0] = 0;
		}
		sandpile.topple();
		sandpile
	}

	pub fn into_grid(self) -> Vec<Vec<u8>> {
		self.grid
	}

	pub fn topple(&mut self) -> u64 {
		let mut excessive = HashSet::new();
		let mut ex2;
		for i in 0..self.grid.len() {
			for j in 0..self.grid[i].len() {
				if self.grid[i][j] >= 4 {
					excessive.insert((i, j));
				}
			}
		}
		let mut count = 0;
		while !excessive.is_empty() {
			ex2 = HashSet::new();
			for c in excessive.drain() {
				let (i, j) = c;
				let d = self.grid[i][j] / 4;
				if d == 0 {
					continue;
				}
				self.grid[i][j] %= 4;
				count += d as u64;
				let mut topple_to = HashSet::new();
				match self.grid_type {
					GridType::Finite => {
						if i > 0 {
							topple_to.insert((i-1, j));
						}
						if j > 0 {
							topple_to.insert((i, j-1));
						}
						if i < self.grid.len()-1 {
							topple_to.insert((i+1, j));
						}
						if j < self.grid[i].len()-1 {
							topple_to.insert((i, j+1));
						}
					},
					GridType::Toroidal => {
						let i1 = if i > 0 {i-1} else {self.grid.len()-1};
						if !(i1 == 0 && j == 0) {
							topple_to.insert((i1, j));
						}
						let j1 = if j > 0 {j-1} else {self.grid[0].len()-1};
						if !(i == 0 && j1 == 0) {
							topple_to.insert((i, j1));
						}
						let i1 = if i < self.grid.len()-1 {i+1} else {0};
						if !(i1 == 0 && j == 0) {
							topple_to.insert((i1, j));
						}
						let j1 = if j < self.grid[i].len()-1 {j+1} else {0};
						if !(i == 0 && j1 == 0) {
							topple_to.insert((i, j1));
						}
					},
				};
				for (ti, tj) in topple_to {
					self.grid[ti][tj] += d;
					if self.grid[ti][tj] >= 4 {
						ex2.insert((ti, tj));
					}
				}
			}
			excessive = ex2;
		}
		count
	}

	pub fn order(&self) -> u64
	{
	// TODO?: учесть, что self может и не быть элементом группы, а только элементом моноида
	// проверяется прибавлением к id
		let mut a = self.clone();
		a.add(self).unwrap();
		let mut count = 1;
		while a != *self {
			a.add(self).unwrap();
			count += 1;
		}
		count
	}
	
	pub fn grid_type(&self) -> GridType {
		self.grid_type
	}
}

const ADD_ERR_MSG: &str = "Attempt to add sandpiles on grids of different sizes.";

pub fn png(grid: &Vec<Vec<u8>>, fname: &str) -> Result<(), io::Error> {
	let colors = [
		[0, 0, 0, 255],
		[64, 128, 0, 255],
		[118, 8, 170, 255],
		[255, 214, 0, 255],
	];
	let mut pixels = vec![0; grid.len() * grid[0].len() * 4];
	let mut p = 0;
	for row in grid {
		for el in row {
			pixels[p..p+4].copy_from_slice(&colors[*el as usize]);
			p += 4;
		}
	}
	repng::encode(File::create(fname)?, grid[0].len() as u32, grid.len() as u32, &pixels)
}

fn fmt_grid(grid: &Vec<Vec<u8>>) -> String {
	let vis = [" ", ".", ":", "&"];
	let mut s = String::new();
	for row in grid {
		for el in row {
			s += vis[*el as usize];
		}
		s += "\n";
	}
	s
}
