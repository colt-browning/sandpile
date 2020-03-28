use std::{
	io,
	fs::File,
	fmt,
	error::Error,
	convert::TryFrom,
};

mod optimized;

pub type Cell = u32;
pub type Grid = Vec<Vec<Cell>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridType {
	Infinite(usize, usize),	// Auto-extending grid with no sink.
	                       	// Origin at given position. No sandpile group.
	Finite(FiniteGridType),
}

impl GridType {
	pub fn finite(&self) -> Result<FiniteGridType, SandpileError> {
		if let GridType::Finite(t) = *self {
			Ok(t)
		} else {
			Err(SandpileError::Infinite)
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FiniteGridType {
	Rectangular,	// Finite rectangular grid with sink all around the grid.
	Toroidal,   	// Toroidal rectangular grid with sink at the top-left node.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Neighbourhood {
	VonNeumann,
	Moore,
}

impl Neighbourhood {
	fn neighbours(&self) -> Cell {
		match *self {
			Neighbourhood::VonNeumann => 4,
			Neighbourhood::Moore => 8,
		}
	}
}

#[derive(Debug, Clone, Hash)]
pub struct GridSandpile {
	grid_type: GridType,
	neighbourhood: Neighbourhood,
	grid: Grid,
	last_topple: u64,
}

#[derive(Debug, Hash)]
pub struct FiniteGridSandpile<'a> {
	grid_type: FiniteGridType,
	neighbourhood: Neighbourhood,
	grid: &'a Grid,
	last_topple: u64,
}

impl<'a> std::convert::AsRef<Grid> for FiniteGridSandpile<'a> {
	fn as_ref(&self) -> &Grid {
		self.grid
	}
}

impl PartialEq for GridSandpile {
	fn eq(&self, other: &GridSandpile) -> bool {
		self.grid_type == other.grid_type && self.neighbourhood == other.neighbourhood && self.grid == other.grid
	}
}

impl Eq for GridSandpile {}

pub const VIS: [char; 9] = [' ', '.', ':', '&', '#', '5', '6', '7', '8'];

impl fmt::Display for GridSandpile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for row in &self.grid {
			for el in row {
				write!(f, "{}", VIS[if *el < 8 {*el} else {8} as usize])?;
			}
			writeln!(f)?;
		}
		Ok(())
	}
}

impl GridSandpile {
	fn verify_rectangular_grid(grid: Grid) -> Result<Grid, SandpileError> {
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
		Ok(grid)
	}

	pub fn from_grid(grid_type: GridType, neighbourhood: Neighbourhood, grid: Grid) -> Result<GridSandpile, SandpileError> {
		let grid = Self::verify_rectangular_grid(grid)?;
		let mut sandpile = GridSandpile {
			grid_type,
			neighbourhood,
			grid,
			last_topple: 0,
		};
		if grid_type == GridType::Infinite(0, 0) && sandpile.grid.len() == 1 && sandpile.grid[0].len() == 1 {
			sandpile.delta00_infinite_optimized();
			return Ok(sandpile)
		}
		sandpile.topple();
		Ok(sandpile)
	}

	pub fn from_string(grid_type: GridType, neighbourhood: Neighbourhood, (x, y): (usize, usize), s: String) -> Result<GridSandpile, SandpileError> {
		let mut g = Vec::new();
		for line in s.lines() {
			let mut row = Vec::new();
			'l: for ch in line.chars() {
				for (n, &vch) in VIS.iter().enumerate() {
					if ch == vch {
						row.push(n as Cell);
						continue 'l
					}
				}
				return Err(SandpileError::UnknownSymbol(ch))
			}
			g.push(row);
		}
		if y == 0 || x == 0 || g.len() == 0 {
			return Err(SandpileError::EmptyGrid);
		}
		if g.len() != y || g[0].len() != x {
			return Err(SandpileError::UnequalDimensions(x, y, g[0].len(), g.len()))
			// actual error might be UnequalRowLengths, but it doesn't matter
		}
		GridSandpile::from_grid(grid_type, neighbourhood, g)
	}

	pub fn add(&mut self, p: &GridSandpile) -> Result<(), SandpileError> {
		if let (GridType::Infinite(mut o1y, mut o1x), GridType::Infinite(o2y, o2x))
		 = (self.grid_type, p.grid_type) {
			if o2x > o1x {
				for row in self.grid.iter_mut() {
					let mut prow = vec![0; o2x-o1x];
					prow.append(row);
					*row = prow;
				}
				o1x = o2x;
			}
			if o2y > o1y {
				let mut pgrid = vec![vec![0; self.grid.len()]; o2y-o1y];
				pgrid.append(&mut self.grid);
				self.grid = pgrid;
				o1y = o2y;
			}
			self.grid_type = GridType::Infinite(o1y, o1x);
			for i in (o1y-o2y)..self.grid.len() {
				if i+o2y-o1y >= p.grid.len() {
					break
				}
				for j in (o1x-o2x)..self.grid[0].len() {
					if j+o2x-o1x >= p.grid[0].len() {
						break
					}
					self.grid[i][j] += p.grid[i+o2y-o1y][j+o2x-o1x];
				}
				if self.grid[i].len() < p.grid[0].len()+o2x-o1x {
					for el in &p.grid[i+o2y-o1y][self.grid[i].len()+o1x-o2x..] {
						self.grid[i].push(*el);
					}
				}
			}
			if self.grid.len() < p.grid.len()+o2y-o1y {
				for row in &p.grid[self.grid.len()+o1y-o2y..] {
					self.grid.push(row.clone());
				}
			}
			self.topple();
			return Ok(())
		}
		if p.grid_type != self.grid_type {
			return Err(SandpileError::UnequalTypes(self.grid_type, p.grid_type));
		}
		if p.grid.len() != self.grid.len() || p.grid[0].len() != self.grid[0].len() {
			return Err(SandpileError::UnequalDimensions(
			self.grid.len(), self.grid[0].len(), p.grid.len(), p.grid[0].len()));
		}
		self.add_grid_unchecked(&p.grid);
		Ok(())
	}
	
	fn add_grid_unchecked(&mut self, pgrid: &Grid) {
		for i in 0..self.grid.len() {
			for j in 0..self.grid[0].len() {
				self.grid[i][j] += pgrid[i][j];
			}
		}
		self.topple();
	}
	
	pub fn into_grid(self) -> Grid {
		self.grid
	}

	fn topple(&mut self) -> u64 {
		if self.grid_type == GridType::Finite(FiniteGridType::Toroidal) {
			self.grid[0][0] = 0;
		}
		let mut excessive = Vec::new();
		let mut ex2;
		for i in 0..self.grid.len() {
			for j in 0..self.grid[i].len() {
				if self.grid[i][j] >= self.neighbourhood.neighbours() {
					excessive.push((i, j));
				}
			}
		}
		let mut count = 0;
		while !excessive.is_empty() {
			ex2 = Vec::new();
			let (mut inc_i, mut inc_j) = (false, false);
			for (i, j) in excessive {
				let i = if inc_i { i+1 } else {i};
				let j = if inc_j { j+1 } else {j};
				let d = self.grid[i][j] / self.neighbourhood.neighbours();
				if d == 0 {
					continue;
				}
				self.grid[i][j] %= self.neighbourhood.neighbours();
				count += d as u64;
				let mut topple_to = Vec::new();
				match self.grid_type {
					GridType::Finite(FiniteGridType::Rectangular) => {
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
						if self.neighbourhood == Neighbourhood::Moore {
							if i > 0 && j > 0 {
								topple_to.push((i-1, j-1));
							}
							if i > 0 && j < self.grid[i].len()-1 {
								topple_to.push((i-1, j+1));
							}
							if i < self.grid.len()-1 && j > 0 {
								topple_to.push((i+1, j-1));
							}
							if i < self.grid.len()-1 && j < self.grid[i].len()-1 {
								topple_to.push((i+1, j+1));
							}
						}
					},
					GridType::Finite(FiniteGridType::Toroidal) => {
						let im1 = if i > 0 {i-1} else {self.grid.len()-1};
						if !(im1 == 0 && j == 0) {
							topple_to.push((im1, j));
						}
						let jm1 = if j > 0 {j-1} else {self.grid[0].len()-1};
						if !(i == 0 && jm1 == 0) {
							topple_to.push((i, jm1));
						}
						let ip1 = if i < self.grid.len()-1 {i+1} else {0};
						if !(ip1 == 0 && j == 0) {
							topple_to.push((ip1, j));
						}
						let jp1 = if j < self.grid[i].len()-1 {j+1} else {0};
						if !(i == 0 && jp1 == 0) {
							topple_to.push((i, jp1));
						}
						if self.neighbourhood == Neighbourhood::Moore {
							if !(im1 == 0 && jm1 == 0) {
								topple_to.push((im1, jm1));
							}
							if !(im1 == 0 && jp1 == 0) {
								topple_to.push((im1, jp1));
							}
							if !(ip1 == 0 && jm1 == 0) {
								topple_to.push((ip1, jm1));
							}
							if !(ip1 == 0 && jp1 == 0) {
								topple_to.push((ip1, jp1));
							}
						}
					},
					GridType::Infinite(oy, ox) => {
						let (mut i, mut j) = (i, j);
						if j == 0 {
							for row in self.grid.iter_mut() {
								row.insert(0, 0);
							}
							for (_, tj) in ex2.iter_mut() {
								*tj += 1;
							}
							j = 1;
							inc_j = true;
							self.grid_type = GridType::Infinite(oy, ox+1);
						}
						if j + 1 == self.grid[0].len() {
							for row in self.grid.iter_mut() {
								row.push(0);
							}
						}
						if i == 0 {
							self.grid.insert(0, vec![0; self.grid[0].len()]);
							for (ti, _) in ex2.iter_mut() {
								*ti += 1;
							}
							i = 1;
							inc_i = true;
							self.grid_type = GridType::Infinite(oy+1, ox);
						}
						if i + 1 == self.grid.len() {
							self.grid.push(vec![0; self.grid[0].len()]);
						}
						topple_to.push((i-1, j));
						topple_to.push((i+1, j));
						topple_to.push((i, j-1));
						topple_to.push((i, j+1));
						if self.neighbourhood == Neighbourhood::Moore {
							topple_to.push((i-1, j-1));
							topple_to.push((i+1, j-1));
							topple_to.push((i-1, j+1));
							topple_to.push((i+1, j+1));
						}
					},
				};
				for (ti, tj) in topple_to {
					self.grid[ti][tj] += d;
					if self.grid[ti][tj] >= self.neighbourhood.neighbours() {
						ex2.push((ti, tj));
					}
				}
			}
			excessive = ex2;
		}
		self.last_topple = count;
		count
	}
	
	pub fn chips_count(&self) -> u64 {
		self.grid.iter().map(|row| -> u64 { row.iter().map(|&x| x as u64).sum() }).sum()
	}
	
	pub fn last_topple(&self) -> u64 {
		self.last_topple
	}
	
	pub fn grid_type(&self) -> GridType {
		self.grid_type
	}
}

impl<'a, 'b: 'a> TryFrom<&'b GridSandpile> for FiniteGridSandpile<'a> {
	type Error = SandpileError;

	fn try_from(s: &'b GridSandpile) -> Result<Self, Self::Error> {
		if let GridType::Finite(grid_type) = s.grid_type {
			Ok(FiniteGridSandpile {
				grid_type,
				neighbourhood: s.neighbourhood,
				grid: &s.grid,
				last_topple: s.last_topple,
			})
		} else {
			Err(SandpileError::Infinite)
		}
	}
}

impl<'a> FiniteGridSandpile<'a> {
	pub fn neutral(grid_type: FiniteGridType, neighbourhood: Neighbourhood, (x, y): (usize, usize)) -> GridSandpile {
		if grid_type == FiniteGridType::Rectangular && neighbourhood == Neighbourhood::VonNeumann && x % 2 == 0 && y == x && x >= 6 {
			return FiniteGridSandpile::neutral_rect_vn_es_optimized(x/2)
		}
	// Proposition 6.36 of http://people.reed.edu/~davidp/divisors_and_sandpiles/
		let t = 2 * (neighbourhood.neighbours() - 1);
		let mut sandpile = GridSandpile::from_grid(GridType::Finite(grid_type), neighbourhood, vec![vec![t; x]; y]).unwrap();
		for row in &mut sandpile.grid {
			for el in row {
				*el = t - *el;
			}
		}
		sandpile.topple();
		sandpile
	}

	pub fn burn(grid_type: FiniteGridType, neighbourhood: Neighbourhood, (x, y): (usize, usize)) -> GridSandpile {
		let mut g = vec![vec![0; x]; y];
		match grid_type {
			FiniteGridType::Rectangular => {
				let border_neighbours = match neighbourhood {
					Neighbourhood::VonNeumann => 1,
					Neighbourhood::Moore => 3,
				};
				for j in 0..x {
					g[0][j] = border_neighbours;
					g[y-1][j] += border_neighbours;
				}
				for i in 0..y {
					g[i][0] += border_neighbours;
					g[i][x-1] += border_neighbours;
				}
				if neighbourhood == Neighbourhood::Moore {
					for &(i, j) in &[(0, 0), (0, x-1), (y-1, 0), (y-1, x-1)] {
						g[i][j] -= 1;
					}
				}
			},
			FiniteGridType::Toroidal => {
				for &(i, j) in &[(0, 1%x), (1%y, 0), (y-1, 0), (0, x-1)] {
					g[i][j] += 1;
				}
				if neighbourhood == Neighbourhood::Moore {
					for &(i, j) in &[(1%y, 1%x), (1%y, x-1), (y-1, 1%x), (y-1, x-1)] {
						g[i][j] += 1;
					}
				}
			},
		};
		GridSandpile::from_grid(GridType::Finite(grid_type), neighbourhood, g).unwrap()
	}

	pub fn inverse(&self) -> GridSandpile {
		let t = 2 * (self.neighbourhood.neighbours() - 1);
		let mut sandpile = GridSandpile::from_grid(GridType::Finite(self.grid_type), self.neighbourhood, vec![vec![t; self.grid[0].len()]; self.grid.len()]).unwrap();
		for y in 0..self.grid.len() {
			for x in 0..self.grid[0].len() {
				sandpile.grid[y][x] = 2 * (t - sandpile.grid[y][x]) - self.grid[y][x];
			}
		}
		sandpile.topple();
		sandpile
	}

	pub fn order(&self) -> u64
	{
		let mut a = GridSandpile::from_grid(GridType::Finite(self.grid_type), self.neighbourhood, self.grid.clone()).unwrap();
		a.add_grid_unchecked(self.grid);
		let mut count = 1;
		while &a.grid != self.grid {
			a.add_grid_unchecked(self.grid);
			count += 1;
		}
		count
	}
}

#[derive(Debug, Hash)]
pub enum SandpileError {
	EmptyGrid,
	EmptyFirstRow(Grid),
	UnequalRowLengths(Grid, usize, usize, usize),
	UnequalTypes(GridType, GridType),
	UnequalDimensions(usize, usize, usize, usize),
	UnknownSymbol(char),
	Infinite,
}

impl fmt::Display for SandpileError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			SandpileError::EmptyGrid => write!(f, "Attempt to build a sandpile upon zero-size grid."),
			SandpileError::EmptyFirstRow(_) => write!(f, "Sandpile grid has empty initial row."),
			SandpileError::UnequalRowLengths(_, expected, n, got) =>
				write!(f, "Sandpile grid does not represent rectangular matrix: initial row has length {}, row {} has length {}.",
					expected, n, got),
			SandpileError::UnequalTypes(expected, got) =>
				write!(f, "Adding sandpiles on grids of different types: {:?} and {:?}.", expected, got),
			SandpileError::UnequalDimensions(self_x, self_y, other_x, other_y) =>
				write!(f, "Incorrect dimensions of sandpile grids: expected {}x{}, got {}x{}.",
					self_x, self_y, other_x, other_y),
			SandpileError::UnknownSymbol(ch) => write!(f, "Unknown symbol in the text representation of a sandpile: {}", ch),
			SandpileError::Infinite => write!(f, "Attempted to view infinite sandpile as finite sandpile."),
		}
	}
}

impl Error for SandpileError {}

impl SandpileError {
	pub fn into_grid(self) -> Option<Grid> {
		match self {
			SandpileError::EmptyFirstRow(grid)
			| SandpileError::UnequalRowLengths(grid, ..) =>
				Some(grid),
			_ => None,
		}
	}
}

pub fn png(grid: &Grid, fname: &str, colors: &Vec<[u8; 4]>) -> io::Result<()> {
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
	fn id_rectangular() {
		let s = FiniteGridSandpile::neutral(FiniteGridType::Rectangular, Neighbourhood::VonNeumann, (3, 2));
		let g = s.into_grid();
		assert_eq!(g, [[2, 1, 2], [2, 1, 2]]);
	}
	
	#[test]
	fn id_rect_optimized() {
		let s = FiniteGridSandpile::neutral(FiniteGridType::Rectangular, Neighbourhood::VonNeumann, (6, 6));
		let g = s.into_grid();
		assert_eq!(g, [
			[2, 1, 3, 3, 1, 2],
			[1, 2, 2, 2, 2, 1],
			[3, 2, 2, 2, 2, 3],
			[3, 2, 2, 2, 2, 3],
			[1, 2, 2, 2, 2, 1],
			[2, 1, 3, 3, 1, 2],
		]);
		let s = FiniteGridSandpile::neutral(FiniteGridType::Rectangular, Neighbourhood::VonNeumann, (10, 10));
		let g = s.into_grid();
		assert_eq!(g, [
			[2, 3, 3, 0, 3, 3, 0, 3, 3, 2],
			[3, 2, 2, 1, 2, 2, 1, 2, 2, 3],
			[3, 2, 2, 3, 3, 3, 3, 2, 2, 3],
			[0, 1, 3, 2, 2, 2, 2, 3, 1, 0],
			[3, 2, 3, 2, 2, 2, 2, 3, 2, 3],
			[3, 2, 3, 2, 2, 2, 2, 3, 2, 3],
			[0, 1, 3, 2, 2, 2, 2, 3, 1, 0],
			[3, 2, 2, 3, 3, 3, 3, 2, 2, 3],
			[3, 2, 2, 1, 2, 2, 1, 2, 2, 3],
			[2, 3, 3, 0, 3, 3, 0, 3, 3, 2],
		]);
	}
	
	#[test]
	fn id_torus() {
		let s = FiniteGridSandpile::neutral(FiniteGridType::Toroidal, Neighbourhood::VonNeumann, (3, 2));
		let g = s.into_grid();
		assert_eq!(g, [[0, 3, 3], [2, 1, 1]]);
	}
	
	#[test]
	fn infinite_delta00() {
		let mut s = GridSandpile {
			grid_type: GridType::Infinite(0, 0),
			neighbourhood: Neighbourhood::VonNeumann,
			grid: vec![vec![16]],
			last_topple: 0,
		};
		s.topple();
		let s2 = GridSandpile::from_grid(GridType::Infinite(0, 0), Neighbourhood::VonNeumann, vec![
			vec![0, 0, 1, 0, 0],
			vec![0, 2, 1, 2, 0],
			vec![1, 1, 0, 1, 1],
			vec![0, 2, 1, 2, 0],
			vec![0, 0, 1, 0, 0],
		]).unwrap();
		assert_eq!(s.grid, s2.grid);
	}
	
	#[test]
	fn infinite_delta00_optimized() {
		let mut s = GridSandpile {
			grid_type: GridType::Infinite(0, 0),
			neighbourhood: Neighbourhood::VonNeumann,
			grid: vec![vec![200]],
			last_topple: 0,
		};
		s.topple();
		let s2 = GridSandpile::from_grid(GridType::Infinite(0, 0), Neighbourhood::VonNeumann, vec![vec![200]]).unwrap();
		assert_eq!(s.grid, s2.grid);
		assert_eq!(s.last_topple, s2.last_topple);
		let mut s = GridSandpile {
			grid_type: GridType::Infinite(0, 0),
			neighbourhood: Neighbourhood::Moore,
			grid: vec![vec![200]],
			last_topple: 0,
		};
		s.topple();
		let s2 = GridSandpile::from_grid(GridType::Infinite(0, 0), Neighbourhood::Moore, vec![vec![200]]).unwrap();
		assert_eq!(s.grid, s2.grid);
		assert_eq!(s.last_topple, s2.last_topple);
	}
	
	#[test]
	fn from_string() {
		let st = "&. \n:.:\n";
		let s = GridSandpile::from_string(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::VonNeumann, (3, 2), String::from(st)).unwrap();
		let g = s.into_grid();
		assert_eq!(g, [[3, 1, 0], [2, 1, 2]]);
		let s = GridSandpile::from_string(GridType::Finite(FiniteGridType::Toroidal), Neighbourhood::VonNeumann, (3, 2), String::from(st)).unwrap();
		let g = s.into_grid();
		assert_eq!(g, [[0, 1, 0], [2, 1, 2]]);
	}
	
	#[test]
	fn display() {
		let g = vec![vec![3, 1, 0], vec![2, 1, 2]];
		let s = GridSandpile::from_grid(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::VonNeumann, g.clone()).unwrap();
		assert_eq!(&s.to_string(), "&. \n:.:\n");
		let s = GridSandpile::from_grid(GridType::Finite(FiniteGridType::Toroidal), Neighbourhood::VonNeumann, g).unwrap();
		assert_eq!(&s.to_string(), " . \n:.:\n");
	}
	
	#[test]
	fn add() {
		let mut s1 = GridSandpile::from_grid(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::VonNeumann, vec![vec![2, 1, 2], vec![3, 3, 1], vec![2, 3, 1]]).unwrap();
		let r = s1.clone();
		let s2 = GridSandpile::from_grid(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::VonNeumann, vec![vec![2, 1, 2], vec![1, 0, 1], vec![2, 1, 2]]).unwrap();
		s1.add(&s2).unwrap();
		assert_eq!(s1, r);
		assert_eq!(r.last_topple(), 0);
		assert_eq!(s1.last_topple(), 9);
	}
	
	#[test]
	fn order() {
		let s = GridSandpile::from_grid(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::VonNeumann, vec![vec![3, 3, 3], vec![3, 3, 3]]).unwrap();
		assert_eq!(FiniteGridSandpile::try_from(&s).unwrap().order(), 7);
	}
	
	#[test]
	fn inverse() {
		let s = GridSandpile::from_grid(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::VonNeumann, vec![vec![3, 3, 3], vec![3, 3, 3]]).unwrap();
		let i = GridSandpile::from_grid(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::VonNeumann, vec![vec![2, 3, 2], vec![2, 3, 2]]).unwrap();
		assert_eq!(FiniteGridSandpile::try_from(&s).unwrap().inverse(), i);
	}
	
	#[test]
	fn moore() {
		assert_eq!(
			GridSandpile::from_grid(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::Moore, vec![vec![0, 0, 0], vec![0, 9, 0], vec![0, 0, 0]]).unwrap(),
			GridSandpile::from_grid(GridType::Finite(FiniteGridType::Rectangular), Neighbourhood::Moore, vec![vec![1, 1, 1], vec![1, 1, 1], vec![1, 1, 1]]).unwrap()
		);
	}
}
