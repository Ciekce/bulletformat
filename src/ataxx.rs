use crate::BulletFormat;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtaxxBoard {
    bbs: [u64; 3],
    score: i16,
    result: u8,
    stm: bool,
    fullm: u16,
    halfm: u8,
    extra: u8,
}

const _RIGHT_SIZE: () = assert!(std::mem::size_of::<AtaxxBoard>() == 32);

impl BulletFormat for AtaxxBoard {
    type FeatureType = (u8, u8);
    const INPUTS: usize = 147;
    const MAX_FEATURES: usize = 49;

    fn score(&self) -> i16 {
        self.score
    }

    fn result(&self) -> f32 {
        f32::from(self.result) / 2.
    }

    fn result_idx(&self) -> usize {
        usize::from(self.result)
    }
}

impl IntoIterator for AtaxxBoard {
    type Item = (u8, u8);
    type IntoIter = AtaxxBoardIter;
    fn into_iter(self) -> Self::IntoIter {
        AtaxxBoardIter {
            board: self,
            stage: 0,
        }
    }
}

pub struct AtaxxBoardIter {
    board: AtaxxBoard,
    stage: usize,
}

impl Iterator for AtaxxBoardIter {
    type Item = (u8, u8);
    fn next(&mut self) -> Option<Self::Item> {
        if self.board.bbs[self.stage] == 0 {
            self.stage += 1;

            if self.stage > 2 {
                return None;
            }
        }

        let sq = self.board.bbs[self.stage].trailing_zeros();
        Some((self.stage as u8, sq as u8))
    }
}

impl std::str::FromStr for AtaxxBoard {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let split: Vec<_> = s.split('|').collect();

        let fen = split[0];
        let score = split[1].trim();
        let wdl = split[2].trim();

        let parts: Vec<&str> = fen.split_whitespace().collect();
        let board_str = parts[0];
        let stm_str = parts[1];

        let stm = stm_str == "b";

        let mut board = Self {stm, ..Default::default()};
        board.halfm = parts.get(2).unwrap_or(&"0").parse().unwrap_or(0);
        board.fullm = parts.get(3).unwrap_or(&"1").parse().unwrap_or(1);

        let mut idx = 0;

        for row in board_str.split('/').rev() {
            for ch in row.chars() {
                match ch {
                    'r' | 'b' | '-' => {
                        let bb = usize::from(ch == 'b') + 2 * usize::from(ch == '-');
                        board.bbs[bb] |= 1 << idx;
                        idx += 1;
                    },
                    '1'..='7' => idx += usize::from(ch as u8 - b'1' + 1),
                    _ => return Err("Unrecognised Character {ch}".to_string()),
                }
            }
        }

        board.score = if let Ok(x) = score.parse::<i16>() {
            x
        } else {
            println!("{s}");
            return Err(String::from("Bad score!"));
        };

        board.result = match wdl {
            "1.0" | "[1.0]" | "1" => 2,
            "0.5" | "[0.5]" | "1/2" => 1,
            "0.0" | "[0.0]" | "0" => 0,
            _ => {
                println!("{s}");
                return Err(String::from("Bad game result!"));
            }
        };

        if stm {
            board.bbs.swap(0, 1);
            board.score = -board.score;
            board.result = 2 - board.result;
        }

        Ok(board)
    }
}

impl AtaxxBoard {
    pub fn stm(&self) -> usize {
        usize::from(self.stm)
    }

    pub fn halfm(&self) -> u8 {
        self.halfm
    }

    pub fn fullm(&self) -> u16 {
        self.fullm
    }
}

impl std::fmt::Display for AtaxxBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut bbs = self.bbs;
        let mut score = self.score;
        let mut result = self.result;

        if self.stm {
            bbs.swap(0, 1);
            score = -score;
            result = 2 - result;
        }

        let mut fen = String::new();

        for i in (0..7).rev() {
            let mut empty = 0;
            for j in 0..7 {
                let sq = 7 * i + j;
                let bit = 1 << sq;
                let pc = usize::from(bit & bbs[0] > 0)
                    + 2 * usize::from(bit & bbs[1] > 0)
                    + 3 * usize::from(bit & bbs[2] > 0);

                if pc == 0 {
                    empty += 1;
                } else {
                    if empty > 0 {
                        fen += empty.to_string().as_str();
                        empty = 0;
                    }
                    fen += [".", "r", "b", "-"][pc];
                }
            }

            if empty > 0 {
                fen += empty.to_string().as_str();
            }

            if i > 0 {
                fen += "/";
            }
        }

        write!(
            f,
            "{fen} {} {} {} | {score} | {:.1}",
            ["r", "b"][self.stm()],
            self.halfm,
            self.fullm,
            f32::from(result) / 2.0,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::AtaxxBoard;

    #[test]
    fn parse() {
        let fens = [
            "6b/2r4/1rr4/1rb2bb/2bb3/7/5bb r 3 11 | -570 | 0.0",
            "6b/7/5r1/3rrrr/4brr/4bbb/3r1bb b 1 14 | 120 | 0.0",
            "r1rr3/1r1r3/2-b-r1/r1bbrrr/2-b-rr/1bbbbbb/1bbbrbb b 1 30 | -840 | 0.0",
            ];

        for fen in fens {
            let board: AtaxxBoard = fen.parse().unwrap();
            assert_eq!(board.to_string(), fen);
        }
    }
}
