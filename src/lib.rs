extern crate repng;

use std::{
	collections::HashSet,
	io,
	fs::File,
	fmt,
	error::Error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridType {
	Finite,		// Finite rectangular grid with sink all around the grid.
	Toroidal,	// Toroidal rectangular grid with sink at the top-left node.
}

#[derive(Debug, Clone)]
pub struct GridSandpile {
	grid_type: GridType,
	grid: Vec<Vec<u8>>,
	last_topple: u64,
}

impl PartialEq for GridSandpile {
	fn eq(&self, other: &GridSandpile) -> bool {
		self.grid_type == other.grid_type && self.grid == other.grid
	}
}

impl Eq for GridSandpile {}

impl fmt::Display for GridSandpile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let vis = [" ", ".", ":", "&", "#"];
		let mut s = String::new();
		for row in &self.grid {
			for el in row {
				s += vis[if *el < 4 {*el} else {4} as usize];
			}
			s += "\n";
		}
		write!(f, "{}", s)
	}
}

impl GridSandpile {
	pub fn from_grid(grid_type: GridType, grid: Vec<Vec<u8>>) -> Result<GridSandpile, SandpileError> {
		if grid.is_empty() {
			return Err(SandpileError::EmptyGrid);
		}
		let l = grid[0].len();
		if l == 0 {
			return Err(SandpileError::EmptyFirstRow(grid));
		}
		for i in 1..grid.len() {
			let l2 = grid[i].len();
			if l2 != l {
				return Err(SandpileError::UnequalRowLengths(grid, l, i, l2));
			}
		}
		let mut sandpile = GridSandpile {
			grid_type,
			grid,
			last_topple: 0,
		};
		if grid_type == GridType::Toroidal {
			sandpile.grid[0][0] = 0;
		}
		sandpile.topple();
		Ok(sandpile)
	}

	pub fn from_string(grid_type: GridType, (x, y): (usize, usize), s: String) -> Result<GridSandpile, SandpileError> {
		let mut g = Vec::new();
		for line in s.lines() {
			let mut row = Vec::new();
			for ch in line.chars() {
				row.push(match ch {
					' ' => 0,
					'.' => 1,
					':' => 2,
					'&' => 3,
					'#' => 4,
					_ => return Err(SandpileError::UnknownSymbol(ch))
				});
			}
			g.push(row);
		}
		if y == 0 || x == 0 || g.len() == 0 {
			return Err(SandpileError::EmptyGrid);
		}
		GridSandpile::from_grid(grid_type, g)
	}

	pub fn add(&mut self, p: &GridSandpile) -> Result<(), SandpileError> {
		if p.grid_type != self.grid_type {
			return Err(SandpileError::UnequalTypes(self.grid_type, p.grid_type));
		}
		if p.grid.len() != self.grid.len() || p.grid[0].len() != self.grid[0].len() {
			return Err(SandpileError::UnequalDimensions(
			self.grid.len(), self.grid[0].len(), p.grid.len(), p.grid[0].len()));
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

	fn topple(&mut self) -> u64 {
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
				let mut topple_to = Vec::new();
				match self.grid_type {
					GridType::Finite => {
						if i > 0 {
							topple_to.push((i-1, j));
						}
						if j > 0 {
							topple_to.push((i, j-1));
						}
						if i < self.grid.len()-1 {
							topple_to.push((i+1, j));
						}
						if j < self.grid[i].len()-1 {
							topple_to.push((i, j+1));
						}
					},
					GridType::Toroidal => {
						let i1 = if i > 0 {i-1} else {self.grid.len()-1};
						if !(i1 == 0 && j == 0) {
							topple_to.push((i1, j));
						}
						let j1 = if j > 0 {j-1} else {self.grid[0].len()-1};
						if !(i == 0 && j1 == 0) {
							topple_to.push((i, j1));
						}
						let i1 = if i < self.grid.len()-1 {i+1} else {0};
						if !(i1 == 0 && j == 0) {
							topple_to.push((i1, j));
						}
						let j1 = if j < self.grid[i].len()-1 {j+1} else {0};
						if !(i == 0 && j1 == 0) {
							topple_to.push((i, j1));
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
		self.last_topple = count;
		count
	}
	
	pub fn last_topple(&self) -> u64 {
		self.last_topple
	}
	
	pub fn inverse(&self) -> GridSandpile {
		let mut sandpile = GridSandpile::from_grid(self.grid_type, vec![vec![6; self.grid[0].len()]; self.grid.len()]).unwrap();
		sandpile.topple();
		for y in 0..self.grid.len() {
			for x in 0..self.grid[0].len() {
				sandpile.grid[y][x] = 2 * (6 - sandpile.grid[y][x]) - self.grid[y][x];
			}
		}
		if self.grid_type == GridType::Toroidal {
			sandpile.grid[0][0] = 0;
		}
		sandpile.topple();
		sandpile
	}

	pub fn order(&self) -> u64
	{
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

#[derive(Debug)]
pub enum SandpileError {
	EmptyGrid,
	EmptyFirstRow(Vec<Vec<u8>>),
	UnequalRowLengths(Vec<Vec<u8>>, usize, usize, usize),
	UnequalTypes(GridType, GridType),
	UnequalDimensions(usize, usize, usize, usize),
	UnknownSymbol(char),
}

impl fmt::Display for SandpileError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			SandpileError::EmptyGrid => write!(f, "Attempt to build a sandpile upon zero-size grid."),
			SandpileError::EmptyFirstRow(_) => write!(f, "Grid has empty initial row."),
			SandpileError::UnequalRowLengths(_, expected, n, got) =>
				write!(f, "Grid (vector of vectors) does not represent rectangular matrix: initial row has length {}, row {} has length {}.",
					expected, n, got),
			SandpileError::UnequalTypes(expected, got) =>
				write!(f, "Adding sandpiles on grids of different types: {:?} and {:?}.", expected, got),
			SandpileError::UnequalDimensions(self_x, self_y, other_x, other_y) =>
				write!(f, "Adding sandpiles on grids of different sizes: {}x{} and {}x{}.",
					self_x, self_y, other_x, other_y),
			SandpileError::UnknownSymbol(ch) => write!(f, "Unknown symbol: {}", ch),
		}
	}
}

impl Error for SandpileError {
	fn description(&self) -> &str {
		match *self {
			SandpileError::EmptyGrid => "empty grid",
			SandpileError::EmptyFirstRow(..) => "empty first row",
			SandpileError::UnequalRowLengths(..) => "unequal row lengths",
			SandpileError::UnequalTypes(..) => "unequal types",
			SandpileError::UnequalDimensions(..) => "unequal dimensions",
			SandpileError::UnknownSymbol(..) => "unknown symbol",
		}
	}
	
	fn cause(&self) -> Option<&dyn Error> {
		None
	}
}

impl SandpileError {
	pub fn into_grid(self) -> Option<Vec<Vec<u8>>> {
		match self {
			SandpileError::EmptyFirstRow(grid)
			| SandpileError::UnequalRowLengths(grid, _, _, _) =>
				Some(grid),
			_ => None,
		}
	}
}

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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn id_finite() {
		let s = GridSandpile::neutral(GridType::Finite, (3, 2));
		let g = s.into_grid();
		assert_eq!(g, vec![vec![2, 1, 2], vec![2, 1, 2]]);
	}
	
	#[test]
	fn id_torus() {
		let s = GridSandpile::neutral(GridType::Toroidal, (3, 2));
		let g = s.into_grid();
		assert_eq!(g, vec![vec![0, 3, 3], vec![2, 1, 1]]);
	}
}
