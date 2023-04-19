mod climber;
mod constant;
pub mod dice;
pub mod error;
mod evaluator;
mod parser;
pub mod roll;
pub mod solver;

#[cfg(test)]
mod tests {
    use crate::roll::Kind;
    use crate::roll::Source;
    use crate::solver::Solver;

    pub struct Mock<'a, T: Iterator<Item = u64>> {
        pub generator: &'a mut T,
    }

    impl<T: Iterator<Item = u64>> Source for Mock<'_, T> {
        fn throw(&mut self, sides: u64) -> u64 {
            match self.generator.next() {
                Some(value) => {
                    if value > sides {
                        panic!("Tried to return `{}` for a `{}` sided dice", value, sides)
                    }
                    println!("Rolled dice `{}`", value);
                    value
                }
                None => panic!("Less values than expected"),
            }
        }
    }

    #[test]
    fn get_repeat_test() {
        let solver = Solver::new("(2d6 + 6) ^ 8 : test").unwrap();
        let mock = vec![3, 5, 3, 5, 3, 5, 3, 5, 3, 5, 3, 5, 3, 5, 3, 5];
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut mock.into_iter(),
            })
            .unwrap();
        match result.get_result() {
            Kind::Single(_) => unreachable!(),
            Kind::Multi(multi) => {
                assert_eq!(8, multi.len());
                for single in multi.iter() {
                    assert_eq!(14, single.get_total());
                }
            }
        }
        for single in result.as_multi().unwrap().iter() {
            eprintln!("{}", single)
        }
        eprintln!("{}", result);
    }

    #[test]
    fn get_repeat_sort_test() {
        let solver = Solver::new("(2d6 + 6) ^# 8 : test").unwrap();
        let mock = vec![3, 5, 1, 1, 6, 5, 3, 5, 4, 5, 2, 4, 3, 5, 1, 2];
        let mut expected = mock
            .as_slice()
            .chunks(2)
            .map(|two| two[0] as i64 + two[1] as i64 + 6)
            .collect::<Vec<_>>();
        expected.sort_unstable();
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut mock.into_iter(),
            })
            .unwrap();
        match result.get_result() {
            Kind::Single(_) => unreachable!(),
            Kind::Multi(multi) => {
                assert_eq!(8, multi.len());
                let results = multi.iter().map(|r| r.get_total()).collect::<Vec<_>>();
                assert_eq!(expected, results);
            }
        };
        eprintln!("{}", result);
    }

    #[test]
    fn get_repeat_sum_test() {
        let solver = Solver::new("(2d6 + 6) ^+ 2 : test").unwrap();
        let mock = vec![3, 5, 4, 2];
        let expected = mock
            .as_slice()
            .chunks(2)
            .map(|two| two[0] as i64 + two[1] as i64 + 6)
            .collect::<Vec<_>>();
        let expected: i64 = expected.iter().sum();
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut mock.into_iter(),
            })
            .unwrap();
        match result.get_result() {
            Kind::Single(_) => unreachable!(),
            Kind::Multi(multi) => {
                assert_eq!(2, multi.len());
                assert_eq!(expected, multi.get_total().unwrap());
            }
        }
        eprintln!("{}", result);
    }

    #[test]
    fn get_single_test() {
        let solver = Solver::new("2d6 + 6 : test").unwrap();
        let mock = vec![3, 5];
        let expected = mock
            .as_slice()
            .chunks(2)
            .map(|two| two[0] as i64 + two[1] as i64)
            .collect::<Vec<_>>();
        let expected = expected.iter().sum::<i64>() + 6;
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut mock.into_iter(),
            })
            .unwrap();
        match result.get_result() {
            Kind::Single(single) => assert_eq!(expected, single.get_total()),
            Kind::Multi(_) => unreachable!(),
        }
        eprintln!("{}", result.as_single().unwrap());
    }

    #[test]
    fn one_value_test() {
        let solver = Solver::new("20").unwrap();
        let result = solver.solve().unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(20, single.get_total());
        } else {
            assert!(false);
        }
    }

    #[test]
    fn one_dice_test() {
        let solver = Solver::new("d20").unwrap();
        let mock = vec![8];
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut mock.into_iter(),
            })
            .unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(8, single.get_total());
        } else {
            unreachable!();
        }
    }

    #[test]
    fn float_mul_test() {
        let solver = Solver::new("20 * 1.5").unwrap();
        let result = solver.solve().unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(30, single.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_signed_mul_test() {
        let solver = Solver::new("20 * +1.5").unwrap();
        let result = solver.solve().unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(30, single.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_neg_signed_mul_test() {
        let solver = Solver::new("20 * -1.5").unwrap();
        let result = solver.solve().unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(-30, single.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_add_test() {
        let solver = Solver::new("20 + 1.5").unwrap();
        let result = solver.solve().unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(21, single.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_signed_add_test() {
        let solver = Solver::new("20 + +1.5").unwrap();
        let result = solver.solve().unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(21, single.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn float_neg_signed_add_test() {
        let solver = Solver::new("20 + -1.5").unwrap();
        let result = solver.solve().unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(18, single.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn signed_add_test() {
        let solver = Solver::new("20 + +5").unwrap();
        let result = solver.solve().unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(25, single.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn signed_neg_add_test() {
        let solver = Solver::new("20 + -5").unwrap();
        let result = solver.solve().unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(15, single.get_total());
        } else {
            unreachable!()
        }
    }

    #[test]
    fn counting_roller_test() {
        let solver = Solver::new("3d6").unwrap();
        let mock = vec![3, 6, 3];
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut mock.into_iter(),
            })
            .unwrap();
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(single.get_total(), 12);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn target_number_test() {
        let solver = Solver::new("10d10 t7").unwrap();
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut (1..11),
            })
            .unwrap();
        println!("{}", result);
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(single.get_total(), 4); // [7, 8, 9, 10] = 4
        } else {
            assert!(false);
        }
    }

    #[test]
    fn target_number_double_test() {
        let solver = Solver::new("10d10 t7 tt9").unwrap();
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut (1..11),
            })
            .unwrap();
        println!("{}", result);
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(single.get_total(), 6); // [7, 8] = 2 and [9, 10] = 4
        } else {
            assert!(false);
        }
    }

    #[test]
    fn target_number_double_lower_than_target_test() {
        // double target > single target
        // single target is ignored
        let solver = Solver::new("10d10 tt7 t9").unwrap();
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut (1..11),
            })
            .unwrap();
        println!("{}", result);
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(single.get_total(), 8); // [7, 8, 9, 10] = 8
        } else {
            assert!(false);
        }
    }

    #[test]
    fn target_number_double_only() {
        let solver = Solver::new("10d10 tt8").unwrap();
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut (1..11),
            })
            .unwrap();
        println!("{}", result);
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(single.get_total(), 6); // [8, 9, 10] = 6
        } else {
            assert!(false);
        }
    }

    #[test]
    fn target_enum() {
        let solver = Solver::new("6d6 t[2,4,6]").unwrap();
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut (1..7),
            })
            .unwrap();
        println!("{}", result);
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(single.get_total(), 3); // [2, 4, 6] = 3
        } else {
            assert!(false);
        }
        let mock = vec![1, 2, 2, 4, 6, 3];
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut mock.into_iter(),
            })
            .unwrap();
        println!("{}", result);
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(single.get_total(), 4); // [2, 4, 6] = 4
        } else {
            assert!(false);
        }
        let mock = vec![1, 3, 3, 4, 6, 3];
        let result = solver
            .solve_with_source(&mut Mock {
                generator: &mut mock.into_iter(),
            })
            .unwrap();
        println!("{}", result);
        let result = result.get_result();
        if let Kind::Single(single) = result {
            assert_eq!(single.get_total(), 2); // [2, 4, 6] = 2
        } else {
            assert!(false);
        }
    }

    #[test]
    fn sandbox_test() {
        let solver = Solver::new("5d6 + 4 * 2").unwrap();
        solver
            .dices()
            .expect("Error while parsing")
            .for_each(|d| eprintln!("{}", d));
        eprintln!("{}\n{}", solver.as_str(), solver.solve().unwrap());
    }
}
